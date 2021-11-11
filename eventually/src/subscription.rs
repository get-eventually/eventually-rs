//! Module containing support for Subscriptions to Events coming
//! from the Event Store.
//!
//! ## What are Subscriptions?
//!
//! Subscriptions, as the name suggest, allows to subscribe to changes
//! in the Event Store. Essentialy, Subscriptions receive Events
//! when they get committed to the Store.
//!
//! This allows for near real-time processing of multiple things, such as
//! publishing committed events on a message broker, or running
//! **projections** (more on that on [`Projection` documentation]).
//!
//! ## Subscriptions in `eventually`
//!
//! ### `EventSubscriber` trait
//!
//! In order to subscribe to Events, `eventually` exposes the
//! [`EventSubscriber`] trait, usually implemented by [`EventStore`]
//! implementations.
//!
//! An [`EventSubscriber`] opens an _endless_ [`EventStream`], that gets
//! closed only at application shutdown, or if the stream gets explicitly
//! dropped.
//!
//! The [`EventStream`] receives all the **new Events** committed
//! to the [`EventStore`].
//!
//! ### `Subscription` trait
//!
//! The [`Subscription`] trait represent an ongoing subscription
//! to Events coming from an [`EventStream`], as described above.
//!
//! Similarly to the [`EventSubscriber`], a [`Subscription`]
//! returns an _endless_ stream of Events called [`SubscriptionStream`].
//!
//! However, [`Subscription`]s are **stateful**: they save the latest
//! Event sequence number that has been processed through the
//! [`SubscriptionStream`], by using the [`checkpoint`] method. Later,
//! the [`Subscription`] can be restarted from where it was left off
//! using the [`resume`] method.
//!
//! This module exposes a simple [`Subscription`] implementation:
//! [`Transient`], for in-memory, one-off subscriptions.
//!
//! For a long-running [`Subscription`] implementation,
//! take a look at persisted subscriptions, such as
//! [`postgres::subscription::Persisted`].
//!
//! [`Projection` documentation]: ../trait.Projection.html
//! [`EventSubscriber`]: trait.EventSubscriber.html
//! [`EventStore`]: ../store/trait.EventStore.html
//! [`EventStream`]: type.EventStream.html
//! [`Subscription`]: trait.Subscription.html
//! [`SubscriptionStream`]: type.SubscriptionStream.html
//! [`checkpoint`]: trait.Subscription.html#tymethod.checkpoint
//! [`resume`]: trait.Subscription.html#tymethod.resume
//! [`Transient`]: struct.Transient.html
//! [`postgres::subscription::Persisted`]:
//! ../postgres/subscription/struct.Persisted.html

use std::error::Error as StdError;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};
use futures::TryFutureExt;

use crate::store::{EventStore, Persisted, Select};

/// Stream of events returned by the [`EventSubscriber::subscribe_all`] method.
pub type EventStream<'a, S> = BoxStream<
    'a,
    Result<
        Persisted<<S as EventSubscriber>::SourceId, <S as EventSubscriber>::Event>,
        <S as EventSubscriber>::Error,
    >,
>;

/// Component to let users subscribe to newly-inserted events into the
/// [`EventStore`].
///
/// Check out [`subscribe_all`](EventSubscriber::subscribe_all) for more
/// information.
///
/// Additional information can be found in the [_Volatile Subscription_] section
/// of eventstore.com
///
/// [_Volatile Subscription_]: https://eventstore.com/docs/getting-started/reading-subscribing-events/index.html#volatile-subscriptions
pub trait EventSubscriber {
    /// Type of the Source id, typically an [`AggregateId`](super::aggregate::AggregateId).
    type SourceId: Eq;

    /// Event type stored in the [`EventStore`], typically an
    /// [`Aggregate::Event`](super::aggregate::Aggregate::Event).
    type Event;

    /// Possible errors returned when receiving events from the notification
    /// channel.
    type Error;

    /// Subscribes to all new events persisted in the [`EventStore`], from
    /// the moment of calling this function, in the future.
    ///
    /// Since this is a long-running stream, make sure not to *block*
    /// or await the full computation of the stream.
    ///
    /// Prefer using a `while let` consumer for this [`EventStream`]:
    ///
    /// ```text
    /// let stream = subscriber.subscribe_all().await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     // Do stuff with the received event...
    /// }
    /// ```
    fn subscribe_all(&self) -> EventStream<Self>;
}

/// Stream of events returned by the [`Subscription::resume`] method.
pub type SubscriptionStream<'a, S> = BoxStream<
    'a,
    Result<
        Persisted<<S as Subscription>::SourceId, <S as Subscription>::Event>,
        <S as Subscription>::Error,
    >,
>;

/// A Subscription to an [`EventStream`] which can be "checkpointed":
/// keeps a record of the latest message processed by itself using
/// [`Subscription::checkpoint`], and can resume working from such message by using the
/// [`Subscription::resume`].
#[async_trait]
pub trait Subscription {
    /// Type of the Source id, typically an [`AggregateId`](super::aggregate::AggregateId).
    type SourceId: Eq;

    /// Event type stored in the [`EventStore`], typically an
    /// [`Aggregate::Event`](super::aggregate::Aggregate::Event).
    type Event;

    /// Possible errors returned when receiving events from the notification
    /// channel.
    type Error;

    /// Resumes the current state of a `Subscription` by returning the
    /// [`EventStream`], starting from the last event processed by the
    /// [`Subscription`].
    fn resume(&self) -> SubscriptionStream<Self>;

    /// Saves the provided version (or sequence number) as the latest
    /// version processed.
    async fn checkpoint(&self, version: u32) -> Result<(), Self::Error>;
}

/// Error type returned by a [`Transient`] Subscription.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error caused by the Subscription's [`EventStore`].
    #[error("error received while listening to the event stream from the store: {0}")]
    Store(#[source] anyhow::Error),

    /// Error caused by the Subscription's [`EventSubscriber`].
    #[error("error received while listening to the event stream subscription: {0}")]
    Subscription(#[source] anyhow::Error),
}

/// [`Subscription`] type which gets deleted once the process using it
/// gets terminated.
///
/// Useful for in-memory or one-off [`Projection`](super::projection::Projection)s.
pub struct Transient<Store, Subscriber> {
    store: Store,
    subscriber: Subscriber,
    last_sequence_number: Arc<AtomicU32>,
}

impl<Store, Subscriber> Transient<Store, Subscriber> {
    /// Creates a new [`Subscription`] using the specified [`EventStore`]
    /// and [`EventSubscriber`] to create the [`SubscriptionStream`] from.
    pub fn new(store: Store, subscriber: Subscriber) -> Self {
        Self {
            store,
            subscriber,
            last_sequence_number: Arc::default(),
        }
    }

    /// Specifies the sequence number of the `Event` the [`SubscriptionStream`]
    /// should start from when calling [`Transient::resume`](Subscription::resume).
    pub fn from(self, sequence_number: u32) -> Self {
        self.last_sequence_number
            .store(sequence_number, Ordering::Relaxed);

        self
    }
}

#[async_trait]
impl<Store, Subscriber> Subscription for Transient<Store, Subscriber>
where
    Store: EventStore + Send + Sync,
    Subscriber: EventSubscriber<
            SourceId = <Store as EventStore>::SourceId,
            Event = <Store as EventStore>::Event,
        > + Send
        + Sync,
    <Store as EventStore>::SourceId: Send + Sync,
    <Store as EventStore>::Event: Send + Sync,
    <Store as EventStore>::Error: StdError + Send + Sync + 'static,
    <Subscriber as EventSubscriber>::Error: StdError + Send + Sync + 'static,
{
    type SourceId = Store::SourceId;
    type Event = Store::Event;
    type Error = Error;

    fn resume(&self) -> SubscriptionStream<Self> {
        let fut = async move {
            // Create the Subscription first, so that once the future has been resolved
            // we'll start receiving events right away.
            //
            // This is to avoid losing events when waiting for the one-off stream
            // to resolve its future.
            //
            // The impact is that we _might_ get duplicated events from the one-off stream
            // and the subscription stream. Luckily, we can discard those by
            // keeping an internal state of the last processed sequence number,
            // and discard all those events that are found.
            let subscription = self.subscriber.subscribe_all();

            let one_off_stream = self
                .store
                .stream_all(Select::From(
                    self.last_sequence_number.load(Ordering::Relaxed),
                ))
                .await
                .map_err(anyhow::Error::from)
                .map_err(Error::Subscription)?;

            let stream = one_off_stream
                .map_err(anyhow::Error::from)
                .map_err(Error::Store)
                .chain(
                    subscription
                        .map_err(anyhow::Error::from)
                        .map_err(Error::Subscription),
                )
                .try_filter_map(move |event| async move {
                    let expected_sequence_number =
                        self.last_sequence_number.load(Ordering::Relaxed);

                    let event_sequence_number = event.sequence_number();

                    if event_sequence_number < expected_sequence_number {
                        #[cfg(feature = "with-tracing")]
                        tracing::trace!(
                            event.sequence_number = event_sequence_number,
                            subscription.sequence_number = expected_sequence_number,
                            "Duplicated event detected; skipping"
                        );

                        return Ok(None);
                    }

                    Ok(Some(event))
                });

            Ok(stream)
        };

        fut.try_flatten_stream().boxed()
    }

    async fn checkpoint(&self, version: u32) -> Result<(), Self::Error> {
        // Checkpointing happens in memory on the atomic sequence number checkpoint.
        self.last_sequence_number.store(version, Ordering::Relaxed);
        Ok(())
    }
}
