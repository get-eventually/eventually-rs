use std::error::Error as StdError;
use std::fmt::Debug;

use async_trait::async_trait;

use futures::stream::BoxStream;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{Event, Events};

pub type PersistedEvents<Id, T> = Vec<PersistedEvent<Id, T>>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PersistedEvent<Id, T> {
    pub(crate) stream_id: Id,
    pub(crate) sequence_number: u32,
    pub(crate) event: Event<T>,
}

impl<Id, T> From<PersistedEvent<Id, T>> for Event<T> {
    #[inline]
    fn from(persisted: PersistedEvent<Id, T>) -> Self {
        persisted.event
    }
}

impl<Id, T> PersistedEvent<Id, T> {
    #[inline]
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    #[inline]
    pub fn stream_id(&self) -> &Id {
        &self.stream_id
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Select {
    All,
    From(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum Version {
    Any,
    Exact(u32),
}

pub trait ConflictError: StdError {
    fn is_conflict(&self) -> bool {
        false
    }
}

impl ConflictError for std::convert::Infallible {}

pub type EventStream<'a, Id, Evt, E> = BoxStream<'a, Result<PersistedEvent<Id, Evt>, E>>;

#[async_trait]
pub trait EventStore<Id, Evt>: Send + Sync {
    type AppendError: ConflictError + Send + Sync;
    type StreamError: StdError + Send + Sync;

    async fn append(
        &mut self,
        id: &Id,
        version: Version,
        events: Events<Evt>,
    ) -> Result<u32, Self::AppendError>;

    fn stream(&self, id: &Id, select: Select) -> EventStream<Id, Evt, Self::StreamError>;

    fn subscribe(&self, id: &Id) -> EventStream<Id, Evt, Self::StreamError>;
}
