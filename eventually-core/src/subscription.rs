//! Module for creating and managing long-running Subscriptions
//! to incoming events in the [`EventStore`].
//!
//! [`EventStore`]: ../store/trait.EventStore.html

use std::error::Error as StdError;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use futures::future::{ok, BoxFuture, FutureExt};
use futures::stream::{BoxStream, StreamExt, TryStreamExt};

use crate::store::{EventStore, Persisted, Select};

/// Stream of events returned by the [`EventSubscriber::subscribe_all`] method.
///
/// [`EventSubscriber::subscribe_all`]:
/// trait.EventSubscriber.html#method.subscribe_all
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
/// [`EventStore`]: ../store/trait.EventStore.html
pub trait EventSubscriber {
    /// Type of the Source id, typically an [`AggregateId`].
    ///
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    type SourceId: Eq;

    /// Event type stored in the [`EventStore`], typically an
    /// [`Aggregate::Event`].
    ///
    /// [`Aggregate::Event`]:
    /// ../aggregate/trait.Aggregate.html#associatedtype.Event
    /// [`EventStore`]: ../store/trait.EventStore.html
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
    ///
    /// [`EventStore`]: ../store/trait.EventStore.html
    /// [`EventStream`]: type.EventStream.html
    fn subscribe_all(&self) -> EventStream<Self>;
}

/// Stream of events returned by the [`Subscription::resume`] method.
///
/// [`Subscription::resume`]: trait.Subscription.html#method.resume
pub type SubscriptionStream<'a, S> = BoxStream<
    'a,
    Result<
        Persisted<<S as Subscription>::SourceId, <S as Subscription>::Event>,
        <S as Subscription>::Error,
    >,
>;

/// A Subscription to an [`EventStream`] which can be "checkpointed":
/// keeps a record of the latest message processed by itself using
/// [`checkpoint`], and can resume working from such message by using the
/// [`resume`].
///
/// [`EventStream`]: type.EventStream.html
/// [`resume`]: trait.Subscription.html#method.resume
/// [`checkpoint`]: trait.Subscription.html#method.checkpoint
pub trait Subscription {
    /// Type of the Source id, typically an [`AggregateId`].
    ///
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    type SourceId: Eq;

    /// Event type stored in the [`EventStore`], typically an
    /// [`Aggregate::Event`].
    ///
    /// [`Aggregate::Event`]:
    /// ../aggregate/trait.Aggregate.html#associatedtype.Event
    /// [`EventStore`]: ../store/trait.EventStore.html
    type Event;

    /// Possible errors returned when receiving events from the notification
    /// channel.
    type Error;

    /// Resumes the current state of a `Subscription` by returning the
    /// [`EventStream`], starting from the last event processed by the
    /// `Subscription`.
    ///
    /// [`EventStream`]: type.EventStream.html
    fn resume(&self) -> BoxFuture<Result<SubscriptionStream<Self>, Self::Error>>;

    /// Saves the provided version (or sequence number) as the latest
    /// version processed.
    fn checkpoint(&self, version: u32) -> BoxFuture<Result<(), Self::Error>>;
}

/// Error type returned by a [`Transient`] Subscription.
///
/// [`Transient`]: struct.Transient.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error caused by the Subscription's [`EventStore`].
    ///
    /// [`EventStore`]: ../store/trait.EventStore.html
    #[error("error received while listening to the event stream from the store: {0}")]
    Store(#[source] anyhow::Error),

    /// Error caused by the Subscription's [`EventSubscriber`].
    ///
    /// [`EventSubscriber`]: trait.EventSubscriber.html
    #[error("error received while listening to the event stream subscription: {0}")]
    Subscription(#[source] anyhow::Error),
}

/// [`Subscription`] type which gets deleted once the process using it
/// gets terminated.
///
/// Useful for in-memory or one-off [`Projection`]s.
///
/// [`Subscription`]: trait.Subscription.html
/// [`Projection`]: ../projection/trait.Projection.html
pub struct Transient<Store, Subscriber> {
    store: Store,
    subscriber: Subscriber,
    last_sequence_number: Arc<AtomicU32>,
}

impl<Store, Subscriber> Transient<Store, Subscriber> {
    /// Creates a new [`Subscription`] using the specified [`EventStore`]
    /// and [`EventSubscriber`] to create the [`SubscriptionStream`] from.
    ///
    /// [`Subscription`]: trait.Subscription.html
    /// [`EventStore`]: ../store/trait.EventStore.html
    /// [`EventSubscriber`]: trait.EventSubscriber.html
    /// [`SubscriptionStream`]: type.SubscriptionStream.html
    pub fn new(store: Store, subscriber: Subscriber) -> Self {
        Self {
            store,
            subscriber,
            last_sequence_number: Default::default(),
        }
    }

    /// Specifies the sequence number of the `Event` the [`SubscriptionStream`]
    /// should start from when calling [`run`].
    ///
    /// [`SubscriptionStream`]: type.SubscriptionStream.html
    /// [`run`]: struct.Transient.html#method.run
    pub fn from(self, sequence_number: u32) -> Self {
        self.last_sequence_number
            .store(sequence_number, Ordering::Relaxed);

        self
    }
}

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

    fn resume(&self) -> BoxFuture<Result<SubscriptionStream<Self>, Self::Error>> {
        Box::pin(async move {
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
                })
                .boxed();

            Ok(stream)
        })
    }

    fn checkpoint(&self, version: u32) -> BoxFuture<Result<(), Self::Error>> {
        // Checkpointing happens in memory on the atomic sequence number checkpoint.
        self.last_sequence_number.store(version, Ordering::Relaxed);
        ok(()).boxed()
    }
}
