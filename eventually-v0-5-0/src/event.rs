use std::fmt::Debug;

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::{
    version::{ToConflictError, Version},
    Message, Messages,
};

pub type Event<T> = Message<T>;
pub type Events<T> = Messages<T>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Persisted<Id, Evt> {
    pub stream_id: Id,
    pub version: Version,
    pub payload: Event<Evt>,
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
    MustBe(Version),
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
