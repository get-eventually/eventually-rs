use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};

use crate::{
    version::{ToConflictError, Version},
    Message, Messages,
};

pub type Event<T> = Message<T>;
pub type Events<T> = Messages<T>;

#[derive(Debug, Clone, PartialEq)]
pub struct Persisted<Id, Evt> {
    pub stream_id: Id,
    pub version: Version,
    pub inner: Event<Evt>,
}

impl<Id, Evt> Persisted<Id, Evt> {
    pub fn map_into<T, U>(self) -> Persisted<T, U>
    where
        Id: Into<T>,
        Evt: Into<U>,
    {
        Persisted {
            stream_id: self.stream_id.into(),
            version: self.version,
            inner: self.inner.map_into(),
        }
    }
}

pub type PersistedEvents<Id, Evt> = Vec<Persisted<Id, Evt>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSelect {
    All,
    From(Version),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamVersionExpected {
    Any,
    Exact(Version),
}

pub type Stream<'a, Id, Evt, Err> = BoxStream<'a, Result<Persisted<Id, Evt>, Err>>;

#[async_trait]
pub trait Store: Send + Sync {
    type StreamId: Send + Sync;
    type Event: Send + Sync;
    type StreamError: Send + Sync;
    type AppendError: ToConflictError + Send + Sync;

    fn stream(
        &self,
        id: &Self::StreamId,
        select: VersionSelect,
    ) -> Stream<Self::StreamId, Self::Event, Self::StreamError>;

    async fn append(
        &self,
        id: Self::StreamId,
        version_check: StreamVersionExpected,
        events: Events<Self::Event>,
    ) -> Result<Version, Self::AppendError>;
}

pub trait StoreExt: Store
where
    Self: Sized,
{
    fn map_into<T, U>(self) -> MappedStore<Self, T, U> {
        MappedStore {
            inner: self,
            id: PhantomData,
            evt: PhantomData,
        }
    }
}

pub struct MappedStore<T, Id, Evt> {
    inner: T,
    id: PhantomData<Id>,
    evt: PhantomData<Evt>,
}

#[async_trait]
impl<T, Id, Evt> Store for MappedStore<T, Id, Evt>
where
    T: Store,
    // for<'a> &'a T::StreamId: TryFrom<&'a Id>,
    // for<'a> <&'a T::StreamId as TryFrom<&'a Id>>::Error: Debug,
    // T::StreamId: TryInto<Id>,
    // <T::StreamId as TryFrom<Id>>::Error: Debug,
    // for<'a> &'a T::StreamId: TryFrom<&'a Id>,
    Id: TryFrom<T::StreamId> + Send + Sync,
    <Id as TryFrom<T::StreamId>>::Error: Debug,
    T::StreamId: From<Id>,
    for<'a> &'a T::StreamId: From<&'a Id>,
    // for<'a> &'a T::StreamId: TryFrom<&'a Id>,
    // for<'a> <&'a T::StreamId as std::convert::TryFrom<&'a Id>>::Error: Debug,
    Evt: TryFrom<T::Event> + Send + Sync,
    <Evt as TryFrom<T::Event>>::Error: Debug,
    T::Event: From<Evt>,
{
    type StreamId = Id;
    type Event = Evt;
    type StreamError = T::StreamError;
    type AppendError = T::AppendError;

    fn stream(
        &self,
        id: &Self::StreamId,
        select: VersionSelect,
    ) -> Stream<Self::StreamId, Self::Event, Self::StreamError> {
        let mapped_id: &T::StreamId = id.into();

        self.inner
            .stream(mapped_id, select)
            .try_filter_map(|evt| async move {
                Ok(Some(Persisted {
                    stream_id: evt.stream_id.try_into().unwrap(),
                    version: evt.version,
                    inner: Message {
                        payload: evt.inner.payload.try_into().unwrap(),
                        metadata: evt.inner.metadata,
                    },
                }))
            })
            .boxed()
    }

    async fn append(
        &self,
        id: Self::StreamId,
        version_check: StreamVersionExpected,
        events: Events<Self::Event>,
    ) -> Result<Version, Self::AppendError> {
        let remapped_id = id.into();
        let remapped_events = events.into_iter().map(Event::map_into).collect();

        self.inner
            .append(remapped_id, version_check, remapped_events)
            .await
    }
}

macro_rules! event_store_test_suite {
    ($expression:expr) => {
        #[cfg(test)]
        mod store_tests {}
    };
}
