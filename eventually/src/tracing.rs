//! Module containing some extension traits to support code instrumentation
//! using the `tracing` crate.

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use async_trait::async_trait;
use tracing::instrument;

use crate::{aggregate, aggregate::Aggregate, event, version::Version};

/// [aggregate::Repository] type wrapper that provides instrumentation
/// features through the `tracing` crate.
#[derive(Debug, Clone)]
pub struct InstrumentedAggregateRepository<T, Inner>
where
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
    Inner: aggregate::Repository<T>,
    <Inner as aggregate::Repository<T>>::Error: Display,
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
    <Inner as aggregate::Repository<T>>::Error: Display,
{
    type Error = Inner::Error;

    #[instrument(name = "aggregate::Repository.get", ret, err, skip(self))]
    async fn get(
        &self,
        id: &T::Id,
    ) -> Result<aggregate::Root<T>, aggregate::RepositoryGetError<Self::Error>> {
        self.inner.get(id).await
    }

    #[instrument(name = "aggregate::Repository.store", ret, err, skip(self))]
    async fn store(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::Error> {
        self.inner.store(root).await
    }
}

/// Extension trait for any [aggregate::Repository] type to provide
/// instrumentation features through the `tracing` crate.
pub trait AggregateRepositoryExt<T>: aggregate::Repository<T> + Sized
where
    Self::Error: Display,
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
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
    <R as aggregate::Repository<T>>::Error: Display,
    T: Aggregate + Debug,
    <T as Aggregate>::Id: Debug,
    <T as Aggregate>::Event: Debug,
{
}

/// [event::Store] type wrapper that provides instrumentation
/// features through the `tracing` crate.
#[derive(Debug, Clone)]
pub struct InstrumentedEventStore<Inner>(Inner)
where
    Inner: event::Store,
    <Inner as event::Store>::StreamId: Debug,
    <Inner as event::Store>::Event: Debug,
    <Inner as event::Store>::AppendError: Display;

#[async_trait]
impl<Inner> event::Store for InstrumentedEventStore<Inner>
where
    Inner: event::Store,
    <Inner as event::Store>::StreamId: Debug,
    <Inner as event::Store>::Event: Debug,
    <Inner as event::Store>::AppendError: Display,
{
    type StreamId = Inner::StreamId;
    type Event = Inner::Event;
    type StreamError = Inner::StreamError;
    type AppendError = Inner::AppendError;

    #[instrument(name = "event::Store.stream", skip(self))]
    fn stream(
        &self,
        id: &Self::StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<Self::StreamId, Self::Event, Self::StreamError> {
        self.0.stream(id, select)
    }

    #[instrument(name = "event::Store.append", ret, err, skip(self))]
    async fn append(
        &self,
        id: Self::StreamId,
        version_check: event::StreamVersionExpected,
        events: Vec<event::Envelope<Self::Event>>,
    ) -> Result<Version, Self::AppendError> {
        self.0.append(id, version_check, events).await
    }
}

/// Extension trait for any [event::Store] type to provide
/// instrumentation features through the `tracing` crate.
pub trait EventStoreExt: event::Store + Sized
where
    <Self as event::Store>::StreamId: Debug,
    <Self as event::Store>::Event: Debug,
    <Self as event::Store>::AppendError: Display,
{
    /// Returns an instrumented version of the [event::Store] instance.
    fn with_tracing(self) -> InstrumentedEventStore<Self> {
        InstrumentedEventStore(self)
    }
}

impl<T> EventStoreExt for T
where
    T: event::Store,
    <T as event::Store>::StreamId: Debug,
    <T as event::Store>::Event: Debug,
    <T as event::Store>::AppendError: Display,
{
}
