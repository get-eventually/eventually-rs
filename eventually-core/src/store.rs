//! Contains the Event Store trait for storing and streaming Aggregate [`Event`]s.
//!
//! [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use serde::{Deserialize, Serialize};

/// An [`Event`] wrapper for events that have been
/// successfully committed to the [`EventStore`].
///
/// [`EventStream`]s are composed of these events.
///
/// [`Event`]: trait.EventStore.html#associatedtype.Event
/// [`EventStream`]: type.EventStream.html
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
    /// Updates the event version to the one specified.
    #[inline]
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Updates the sequence number version to the one specified.
    #[inline]
    pub fn with_sequence_number(mut self, sequence_number: u32) -> Self {
        self.sequence_number = sequence_number;
        self
    }

    /// Returns the event version.
    #[inline]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns the event sequence number.
    #[inline]
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    /// Unwraps the inner [`Event`] from the `PersistedEvent` wrapper.
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    #[inline]
    pub fn take(self) -> T {
        self.event
    }
}

/// Selection operation for the events to capture in an [`EventStream`].
///
/// [`EventStream`]: type.EventStream.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Select {
    /// To return all the [`Event`]s in the [`EventStream`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`EventStream`]: type.EventStream.html
    All,

    /// To return a slice of the [`EventStream`], starting from
    /// those [`Event`]s with version **greater or equal** than
    /// the one specified in this variant.
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`EventStream`]: type.EventStream.html
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
    /// The desired version for the new [`Event`]s to append must be specified.
    ///
    /// Implementations could decide to return an error if the expected
    /// version is different from the one supplied in the method invocation.
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
    /// [`Select`] specifies the selection strategy for the [`Event`]s
    /// in the returned [`EventStream`]: take a look at type documentation
    /// for all the available options.
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    /// [`Select`]: enum.Select.html
    /// [`EventStream`]: type.EventStream.html
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
