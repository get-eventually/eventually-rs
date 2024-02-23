//! Module `event` contains types and abstractions helpful for working
//! with Domain Events.

pub mod store;
use std::fmt::Debug;

use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

pub use crate::event::store::Store;
use crate::{message, version};

/// An Event is a [Message][message::Message] carring the information about a Domain Event,
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
    /// Check the [Version][version::Version] type and module documentation for more info.
    pub version: version::Version,

    /// The actual Domain Event carried by this envelope.
    pub event: Envelope<Evt>,
}

/// Specifies the slice of the Event Stream to select when calling [`Store::stream`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSelect {
    /// Selects all [Event][Envelope]s in the Event [Stream].
    All,

    /// Selects all [Event][Envelope]s in the Event [Stream] starting from the [Event]
    /// with the specified [Version][version::Version].
    From(version::Version),
}

/// Stream is a stream of [Persisted] Domain Events.
pub type Stream<'a, Id, Evt, Err> = BoxStream<'a, Result<Persisted<Id, Evt>, Err>>;
