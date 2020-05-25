//! Contains the Event Store trait for storing and streaming Aggregate [`Event`]s.
//!
//! [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PersistedEvent<T> {
    version: u32,
    sequence_number: u32,
    #[serde(flatten)]
    event: T,
}

impl<T> From<T> for PersistedEvent<T> {
    #[inline]
    fn from(event: T) -> Self {
        Self {
            event,
            version: 0,
            sequence_number: 0,
        }
    }
}

impl<T> PersistedEvent<T> {
    #[inline]
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    #[inline]
    pub fn with_sequence_number(mut self, sequence_number: u32) -> Self {
        self.sequence_number = sequence_number;
        self
    }

    #[inline]
    pub fn version(&self) -> u32 {
        self.version
    }

    #[inline]
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    #[inline]
    pub fn take(self) -> T {
        self.event
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Select {
    All,
    From(u32),
}

/// Stream type returned by the [`EventStore::stream`] method.
///
/// [`EventStore::stream`]: trait.EventStore.html#method.stream
pub type EventStream<'a, S> =
    BoxStream<'a, Result<PersistedEvent<<S as EventStore>::Event>, <S as EventStore>::Error>>;

/// An Event Store is an append-only, ordered list of [`Event`]s
/// for a certain "source" -- e.g. an [`Aggregate`].
///
/// [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
pub trait EventStore {
    /// Type of the Source id, typically an [`AggregateId`].
    ///
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    type SourceId: Eq;

    /// Event to be stored in the `EventStore`, typically an [`Aggregate::Event`].
    ///
    /// [`Aggregate::Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
    type Event;

    /// Possible errors returned by the `EventStore` when requesting operations.
    type Error;

    /// Appends a new list of [`Event`]s to the Event Store, for the Source
    /// entity specified by [`SourceId`].
    ///
    /// `append` is a transactional operation: it either appends all the events,
    /// or none at all and returns an appropriate [`Error`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    /// [`Error`]: trait.EventStore.html#associatedtype.Error
    fn append(
        &mut self,
        id: Self::SourceId,
        version: u32,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<(), Self::Error>>;

    /// Streams a list of [`Event`]s from the `EventStore` back to the application,
    /// by specifying the desired [`SourceId`] and [`Offset`].
    ///
    /// [`SourceId`] will be used to request a particular `EventStream`.
    ///
    /// [`Offset`] will be used to specify a slice of the [`Event`]s to retrieve
    /// from the `EventStore`. To request the whole list, use the [`Default`]
    /// value for [`Offset`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    /// [`Offset`]: trait.EventStore.html#associatedtype.Offset
    fn stream(
        &self,
        id: Self::SourceId,
        select: Select,
    ) -> BoxFuture<Result<EventStream<Self>, Self::Error>>;

    /// Drops all the [`Event`]s related to one `Source`, specified by
    /// the provided [`SourceId`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>>;
}
