//! Module for creating and managing long-running Subscriptions
//! to incoming events in the [`EventStore`].
//!
//! [`EventStore`]: ../store/trait.EventStore.html

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use crate::store::Persisted;

/// Stream of events returned by the [`EventSubscriber::subscribe_all`] method.
///
/// [`EventSubscriber::subscribe_all`]: trait.EventSubscriber.html#method.subscribe_all
pub type EventStream<'a, S> = BoxStream<
    'a,
    Result<
        Persisted<<S as EventSubscriber>::SourceId, <S as EventSubscriber>::Event>,
        <S as EventSubscriber>::Error,
    >,
>;

/// Component to let users subscribe to newly-inserted events into the [`EventStore`].
///
/// Check out [`subscribe_all`] for more information.
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

    /// Event type stored in the [`EventStore`], typically an [`Aggregate::Event`].
    ///
    /// [`Aggregate::Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
    /// [`EventStore`]: ../store/trait.EventStore.html
    type Event;

    /// Possible errors returned when receiving events from the notification channel.
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
    fn subscribe_all(&self) -> BoxFuture<Result<EventStream<Self>, Self::Error>>;
}
