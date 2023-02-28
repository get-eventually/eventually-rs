//! Module containing some extension traits to support code instrumentation
//! using the `tracing` crate.

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use async_trait::async_trait;
use tracing::instrument;

use crate::{
    aggregate,
    aggregate::Aggregate,
    entity::{self, Entity, Identifiable},
    event, message,
    version::Version,
};

/// [entity::Repository] type wrapper that provides instrumentation
/// features through the `tracing` crate.
#[derive(Debug, Clone)]
pub struct InstrumentedAggregateRepository<T, Inner>
where
    T: Entity + Debug,
    <T as Identifiable>::Id: Debug,
    Inner: entity::Repository<T>,
    <Inner as entity::Getter<T>>::Error: Display,
    <Inner as entity::Saver<T>>::Error: Display,
{
    inner: Inner,
    t: PhantomData<T>,
}

#[async_trait]
impl<T, Inner> entity::Getter<T> for InstrumentedAggregateRepository<T, Inner>
where
    T: Entity + Debug,
    <T as Identifiable>::Id: Debug,
    Inner: entity::Repository<T>,
    <Inner as entity::Getter<T>>::Error: Display,
    <Inner as entity::Saver<T>>::Error: Display,
{
    type Error = <Inner as entity::Getter<T>>::Error;

    #[instrument(name = "entity::Repository.get", ret, err, skip(self))]
    async fn get(&self, id: &T::Id) -> Result<T, entity::GetError<Self::Error>> {
        self.inner.get(id).await
    }
}

#[async_trait]
impl<T, Inner> entity::Saver<T> for InstrumentedAggregateRepository<T, Inner>
where
    T: Entity + Debug,
    <T as Identifiable>::Id: Debug,
    Inner: entity::Repository<T>,
    <Inner as entity::Getter<T>>::Error: Display,
    <Inner as entity::Saver<T>>::Error: Display,
{
    type Error = <Inner as entity::Saver<T>>::Error;

    #[instrument(name = "entity::Repository.save", ret, err, skip(self))]
    async fn save(&self, entity: &mut T) -> Result<(), Self::Error> {
        self.inner.save(entity).await
    }
}

/// Extension trait for any [entity::Repository] type to provide
/// instrumentation features through the `tracing` crate.
pub trait EntityRepositoryExt<T>: entity::Repository<T> + Sized
where
    T: Entity + Debug,
    <T as Identifiable>::Id: Debug,
    <Self as entity::Getter<T>>::Error: Display,
    <Self as entity::Saver<T>>::Error: Display,
{
    /// Returns an instrumented version of the [entity::Repository] instance.
    fn with_tracing(self) -> InstrumentedAggregateRepository<T, Self> {
        InstrumentedAggregateRepository {
            inner: self,
            t: PhantomData,
        }
    }
}

impl<R, T> EntityRepositoryExt<T> for R
where
    R: entity::Repository<T>,
    <R as entity::Getter<T>>::Error: Display,
    <R as entity::Saver<T>>::Error: Display,
    T: Entity + Debug,
    <T as Identifiable>::Id: Debug,
{
}

/// [event::Store] type wrapper that provides instrumentation
/// features through the `tracing` crate.
#[derive(Debug, Clone)]
pub struct InstrumentedEventStore<T, StreamId, Event>
where
    T: event::Store<StreamId, Event> + Send + Sync,
    <T as event::Appender<StreamId, Event>>::Error: Display + Send + Sync,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    store: T,
    stream_id: PhantomData<StreamId>,
    event: PhantomData<Event>,
}

impl<T, StreamId, Event> event::Streamer<StreamId, Event>
    for InstrumentedEventStore<T, StreamId, Event>
where
    T: event::Store<StreamId, Event> + Send + Sync,
    <T as event::Appender<StreamId, Event>>::Error: Display + Send + Sync,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    type Error = <T as event::Streamer<StreamId, Event>>::Error;

    #[instrument(name = "event::Store.stream", skip(self))]
    fn stream(
        &self,
        id: &StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<StreamId, Event, Self::Error> {
        self.store.stream(id, select)
    }
}

#[async_trait]
impl<T, StreamId, Event> event::Appender<StreamId, Event>
    for InstrumentedEventStore<T, StreamId, Event>
where
    T: event::Store<StreamId, Event> + Send + Sync,
    <T as event::Appender<StreamId, Event>>::Error: Display + Send + Sync,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    type Error = <T as event::Appender<StreamId, Event>>::Error;

    #[instrument(name = "event::Store.append", ret, err, skip(self))]
    async fn append(
        &self,
        id: StreamId,
        version_check: event::StreamVersionExpected,
        events: Vec<event::Envelope<Event>>,
    ) -> Result<Version, Self::Error> {
        self.store.append(id, version_check, events).await
    }
}

/// Extension trait for any [event::Store] type to provide
/// instrumentation features through the `tracing` crate.
pub trait EventStoreExt<StreamId, Event>: event::Store<StreamId, Event> + Sized
where
    <Self as event::Appender<StreamId, Event>>::Error: Display,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    /// Returns an instrumented version of the [event::Store] instance.
    fn with_tracing(self) -> InstrumentedEventStore<Self, StreamId, Event> {
        InstrumentedEventStore {
            store: self,
            stream_id: PhantomData,
            event: PhantomData,
        }
    }
}

impl<T, StreamId, Event> EventStoreExt<StreamId, Event> for T
where
    T: event::Store<StreamId, Event> + Send + Sync,
    <T as event::Appender<StreamId, Event>>::Error: Display + Send + Sync,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
}
