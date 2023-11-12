//! Module `event` contains types and abstractions helpful for working
//! with Domain Events.

use std::fmt::Debug;

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::message;
use crate::version::{ConflictError, Version};

pub mod store;

/// An Event is a [Message] carring the information about a Domain Event,
/// an occurrence in the system lifetime that is relevant for the Domain
/// that is being implemented.
pub type Envelope<T> = message::Envelope<T>;

/// An [Event] that has been persisted to the Event [Store].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Persisted<Id, Evt>
where
    Evt: message::Message,
{
    /// The id of the Event Stream the persisted Event belongs to.
    pub stream_id: Id,

    /// The version of the Event Stream when this Event has been recorded.
    ///
    /// This value is used for optimistic concurrency checks, to avoid
    /// data races in parallel command evaluations.
    ///
    /// Check the [Version] type and module documentation for more info.
    pub version: Version,

    /// The actual Domain Event carried by this envelope.
    pub event: Envelope<Evt>,
}

/// Specifies the slice of the Event Stream to select when calling [`Store::stream`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSelect {
    /// Selects all [Events] in the Event [Stream].
    All,

    /// Selects all [Events] in the Event [Stream] starting from the [Event]
    /// with the specified [Version].
    From(Version),
}

/// Specifies an expectation on the Event [Stream] version targeted
/// when calling [`Store::append`].
///
/// This type allows for optimistic concurrency checks, avoiding data races
/// when modifying the same Event Stream at the same time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamVersionExpected {
    /// Disables any kind of optimistic concurrency check, instructing the [Store]
    /// to append the new [Event] no matter the current [Version] of the [Stream].
    Any,

    /// Sets the expectation that the Event [Stream] must be at the specified
    /// [Version] for the [`Store::append`] call to succeed.
    MustBe(Version),
}

/// Stream is a stream of [Persisted] Domain Events.
pub type Stream<'a, Id, Evt, Err> = BoxStream<'a, Result<Persisted<Id, Evt>, Err>>;

/// Interface used to stream [Persisted] Domain Events from an Event Store to an application.
pub trait Streamer<StreamId, Event>: Send + Sync
where
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
    /// The error type returned by the Store during a [`stream`] call.
    type Error: Send + Sync;

    /// Opens an Event Stream, effectively streaming all Domain Events
    /// of an Event Stream back in the application.
    fn stream(&self, id: &StreamId, select: VersionSelect) -> Stream<StreamId, Event, Self::Error>;
}

#[async_trait]
/// Interface used to append new Domain Events in an Event Store.
pub trait Appender<StreamId, Event>: Send + Sync
where
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
    /// The error type returned by the Store during an [`append`] call.
    /// It could be a [version::ConflictError], which is why the bound to
    /// `Into<Option<ConflictError>>` is required.
    type Error: Into<Option<ConflictError>> + Send + Sync;

    /// Appens new Domain Events to the specified Event Stream.
    ///
    /// The result of this operation is the new [Version] of the Event Stream
    /// with the specified Domain Events added to it.
    async fn append(
        &self,
        id: StreamId,
        version_check: StreamVersionExpected,
        events: Vec<Envelope<Event>>,
    ) -> Result<Version, Self::Error>;
}

/// An [Event] Store, used to store Domain Events in Event Streams -- a stream
/// of Domain Events -- and retrieve them.
///
/// Each Event Stream is represented by a unique Stream identifier.
pub trait Store<StreamId, Event>:
    Streamer<StreamId, Event> + Appender<StreamId, Event> + Send + Sync
where
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
}

impl<T, StreamId, Event> Store<StreamId, Event> for T
where
    T: Streamer<StreamId, Event> + Appender<StreamId, Event> + Send + Sync,
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
}
