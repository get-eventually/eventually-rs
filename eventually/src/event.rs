//! Module `event` contains types and abstractions helpful for working
//! with Domain Events.

use std::fmt::Debug;

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::{
    version::{ConflictError, Version},
    Message,
};

/// An Event is a [Message] carring the information about a Domain Event,
/// an occurrence in the system lifetime that is relevant for the Domain
/// that is being implemented.
pub type Event<T> = Message<T>;

/// An [Event] that has been persisted to the Event [Store].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Persisted<Id, Evt> {
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
    pub payload: Event<Evt>,
}

/// Shortcut type to represent multiple [Persisted] Events.
pub type PersistedEvents<Id, Evt> = Vec<Persisted<Id, Evt>>;

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

/// An [Event] Store, used to store Domain Events in Event Streams -- a stream
/// of Domain Events -- and retrieve them.
///
/// Each Event Stream is represented by a unique [`Store::StreamId`].
#[async_trait]
pub trait Store: Send + Sync {
    /// The type used to uniquely identify each Event Stream recorded
    /// by the Event Store.
    type StreamId: Send + Sync;

    /// The type containing all Domain Events recorded by the Event Store.
    /// Typically, this parameter should be an `enum`.
    type Event: Send + Sync;

    /// The error type returned by the Store during a [`stream`] call.
    type StreamError: Send + Sync;

    /// The error type returned by the Store during an [`append`] call.
    /// It could be a [version::ConflictError], which is why the bound to
    /// `Into<Option<ConflictError>>` is required.
    type AppendError: Into<Option<ConflictError>> + Send + Sync;

    /// Opens an Event Stream, effectively streaming all Domain Events
    /// of an Event Stream back in the application.
    fn stream(
        &self,
        id: &Self::StreamId,
        select: VersionSelect,
    ) -> Stream<Self::StreamId, Self::Event, Self::StreamError>;

    /// Appens new Domain Events to the specified Event Stream.
    ///
    /// The result of this operation is the new [Version] of the Event Stream
    /// with the specified Domain Events added to it.
    async fn append(
        &self,
        id: Self::StreamId,
        version_check: StreamVersionExpected,
        events: Vec<Event<Self::Event>>,
    ) -> Result<Version, Self::AppendError>;
}
