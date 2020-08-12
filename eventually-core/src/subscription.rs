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

    fn subscribe_all(&self) -> BoxFuture<Result<EventStream<Self>, Self::Error>>;
}
