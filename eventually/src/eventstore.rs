use std::error::Error as StdError;

use async_trait::async_trait;

use futures::stream::BoxStream;

use serde::Serialize;

use crate::{Event, Events, MetadataValue};

pub const GLOBAL_SEQUENCE_NUMBER_METADATA_KEY: &str = "Global-Sequence-Number";

pub fn global_sequence_number<T>(event: &Event<T>) -> Option<i64> {
    event
        .metadata
        .get(GLOBAL_SEQUENCE_NUMBER_METADATA_KEY)
        .and_then(|v| match v {
            MetadataValue::Integer(i) => Some(*i),
            _ => None,
        })
}

pub fn with_global_sequence_number<T>(mut event: Event<T>, sequence_number: i64) -> Event<T> {
    event.metadata.insert(
        GLOBAL_SEQUENCE_NUMBER_METADATA_KEY.to_owned(),
        MetadataValue::Integer(sequence_number),
    );

    event
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersistedEvent<T> {
    pub stream_category: String,
    pub stream_id: String,
    pub version: i64,
    #[serde(flatten)]
    pub event: Event<T>,
}

pub enum Select {
    All,
    From(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream<'a> {
    All,
    Category(&'a str),
    Id { category: &'a str, id: &'a str },
}

#[derive(Debug, Clone, Copy)]
pub enum VersionCheck {
    Any,
    Exact(i64),
}

impl VersionCheck {
    #[inline]
    pub fn check(&self, current_version: i64) -> Result<(), ConflictError> {
        match *self {
            VersionCheck::Any => Ok(()),
            VersionCheck::Exact(expected) if current_version == expected => Ok(()),
            VersionCheck::Exact(expected) => Err(ConflictError {
                expected,
                actual: current_version,
            }),
        }
    }
}

pub trait IntoConflictError {
    fn into_conflict_error(&self) -> Option<ConflictError>;
}

impl IntoConflictError for std::convert::Infallible {
    #[inline]
    fn into_conflict_error(&self) -> Option<ConflictError> {
        None
    }
}

impl IntoConflictError for ConflictError {
    #[inline]
    fn into_conflict_error(&self) -> Option<ConflictError> {
        Some(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("conflict error: expected stream version {expected}, actual {actual}")]
pub struct ConflictError {
    pub expected: i64,
    pub actual: i64,
}

pub type EventStream<'a, T, E> = BoxStream<'a, Result<PersistedEvent<T>, E>>;

#[async_trait]
pub trait EventStore: Sync + Send {
    type Event: Send + Sync;
    type AppendError: StdError + IntoConflictError + Send + Sync + 'static;
    type StreamError: StdError + Send + Sync + 'static;

    async fn append(
        &mut self,
        category: &str,
        stream_id: &str,
        expected: VersionCheck,
        events: Events<Self::Event>,
    ) -> Result<i64, Self::AppendError>;

    fn stream(&self, stream: Stream, select: Select)
        -> EventStream<Self::Event, Self::StreamError>;

    fn subscribe(&self, stream: Stream) -> EventStream<Self::Event, Self::StreamError>;
}
