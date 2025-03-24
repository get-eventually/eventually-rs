//! Module containing some extension traits to support code instrumentation
//! using the `tracing` crate.

use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use tracing::instrument;

use crate::aggregate::Aggregate;
use crate::version::{self, Version};
use crate::{aggregate, event, message};

/// [`aggregate::Repository`] type wrapper that provides instrumentation
/// features through the `tracing` crate.
#[derive(Debug, Clone)]
pub struct InstrumentedAggregateRepository<T, Inner>
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
    Inner: aggregate::Repository<T>,
{
    inner: Inner,
    t: PhantomData<T>,
}

#[async_trait]
impl<T, Inner> aggregate::repository::Getter<T> for InstrumentedAggregateRepository<T, Inner>
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
    Inner: aggregate::Repository<T>,
{
    #[allow(clippy::blocks_in_conditions)] // NOTE(ar3s3ru): seems to be a false positive.
    #[instrument(name = "aggregate::repository::Getter.get", ret, err, skip(self))]
    async fn get(&self, id: &T::Id) -> Result<aggregate::Root<T>, aggregate::repository::GetError> {
        self.inner.get(id).await
    }
}

#[async_trait]
impl<T, Inner> aggregate::repository::Saver<T> for InstrumentedAggregateRepository<T, Inner>
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
    Inner: aggregate::Repository<T>,
{
    #[allow(clippy::blocks_in_conditions)] // NOTE(ar3s3ru): seems to be a false positive.
    #[instrument(name = "aggregate::repository::Saver.save", ret, err, skip(self))]
    async fn save(
        &self,
        root: &mut aggregate::Root<T>,
    ) -> Result<(), aggregate::repository::SaveError> {
        self.inner.save(root).await
    }
}

/// Extension trait for any [`aggregate::Repository`] type to provide
/// instrumentation features through the `tracing` crate.
pub trait AggregateRepositoryExt<T>: aggregate::Repository<T> + Sized
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
{
    /// Returns an instrumented version of the [`aggregate::Repository`] instance.
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
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
{
}

/// [`event::Store`] type wrapper that provides instrumentation
/// features through the `tracing` crate.
#[derive(Debug, Clone)]
pub struct InstrumentedEventStore<T, StreamId, Event>
where
    T: event::Store<StreamId, Event> + Send + Sync,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    store: T,
    stream_id: PhantomData<StreamId>,
    event: PhantomData<Event>,
}

impl<T, StreamId, Event> event::store::Streamer<StreamId, Event>
    for InstrumentedEventStore<T, StreamId, Event>
where
    T: event::Store<StreamId, Event> + Send + Sync,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    type Error = <T as event::store::Streamer<StreamId, Event>>::Error;

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
impl<T, StreamId, Event> event::store::Appender<StreamId, Event>
    for InstrumentedEventStore<T, StreamId, Event>
where
    T: event::Store<StreamId, Event> + Send + Sync,
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    #[allow(clippy::blocks_in_conditions)] // NOTE(ar3s3ru): seems to be a false positive.
    #[instrument(name = "event::Store.append", ret, err, skip(self))]
    async fn append(
        &self,
        id: StreamId,
        version_check: version::Check,
        events: Vec<event::Envelope<Event>>,
    ) -> Result<Version, event::store::AppendError> {
        self.store.append(id, version_check, events).await
    }
}

/// Extension trait for any [`event::Store`] type to provide
/// instrumentation features through the `tracing` crate.
pub trait EventStoreExt<StreamId, Event>: event::Store<StreamId, Event> + Sized
where
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
    /// Returns an instrumented version of the [`event::Store`] instance.
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
    StreamId: Debug + Send + Sync,
    Event: message::Message + Debug + Send + Sync,
{
}
