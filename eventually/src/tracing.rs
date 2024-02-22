//! Module containing some extension traits to support code instrumentation
//! using the `tracing` crate.

use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use async_trait::async_trait;
use tracing::instrument;

use crate::aggregate::Aggregate;
use crate::version::{self, Version};
use crate::{aggregate, event, message};

/// [aggregate::Repository] type wrapper that provides instrumentation
/// features through the `tracing` crate.
#[derive(Debug, Clone)]
pub struct InstrumentedAggregateRepository<T, Inner>
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
    Inner: aggregate::Repository<T>,
    <Inner as aggregate::Repository<T>>::GetError: Display,
    <Inner as aggregate::Repository<T>>::SaveError: Display,
{
    inner: Inner,
    t: PhantomData<T>,
}

#[async_trait]
impl<T, Inner> aggregate::Repository<T> for InstrumentedAggregateRepository<T, Inner>
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
    Inner: aggregate::Repository<T>,
    <Inner as aggregate::Repository<T>>::GetError: Display,
    <Inner as aggregate::Repository<T>>::SaveError: Display,
{
    type GetError = <Inner as aggregate::Repository<T>>::GetError;
    type SaveError = <Inner as aggregate::Repository<T>>::SaveError;

    #[instrument(name = "aggregate::Repository.get", ret, err, skip(self))]
    async fn get(
        &self,
        id: &T::Id,
    ) -> Result<aggregate::Root<T>, aggregate::repository::GetError<Self::GetError>> {
        self.inner.get(id).await
    }

    #[instrument(name = "aggregate::Repository.save", ret, err, skip(self))]
    async fn save(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::SaveError> {
        self.inner.save(root).await
    }
}

/// Extension trait for any [aggregate::Repository] type to provide
/// instrumentation features through the `tracing` crate.
pub trait AggregateRepositoryExt<T>: aggregate::Repository<T> + Sized
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
    <Self as aggregate::Repository<T>>::GetError: Display,
    <Self as aggregate::Repository<T>>::SaveError: Display,
{
    /// Returns an instrumented version of the [aggregate::Repository] instance.
    fn with_tracing(self) -> InstrumentedAggregateRepository<T, Self> {
        InstrumentedAggregateRepository {
            inner: self,
            t: PhantomData,
        }
    }
}

impl<R, T> AggregateRepositoryExt<T> for R
where
    R: aggregate::Repository<T>,
    <R as aggregate::Repository<T>>::GetError: Display,
    <R as aggregate::Repository<T>>::SaveError: Display,
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
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
        version_check: version::Check,
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
