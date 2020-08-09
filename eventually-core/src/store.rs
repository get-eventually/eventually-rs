//! Contains the Event Store trait for storing and streaming Aggregate [`Event`]s.
//!
//! [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event

use std::ops::Deref;

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use serde::{Deserialize, Serialize};

use crate::versioning::Versioned;

/// Contains a type-state builder for [`PersistentEvent`] type.
///
/// [`PersistentEvent`]: struct.PersistedEvent.html
pub mod persistent {
    /// Creates a new [`PersistedEvent`] by wrapping an Event value.
    ///
    /// [`PersistentEvent`]: ../struct.PersistedEvent.html
    pub struct EventBuilder<SourceId, T> {
        pub(super) event: T,
        pub(super) source_id: SourceId,
    }

    impl<SourceId, T> From<(SourceId, T)> for EventBuilder<SourceId, T> {
        #[inline]
        fn from(value: (SourceId, T)) -> Self {
            let (source_id, event) = value;
            Self { source_id, event }
        }
    }

    impl<SourceId, T> EventBuilder<SourceId, T> {
        /// Specifies the [`PersistentEvent`] version and moves to the next
        /// builder state.
        ///
        /// [`PersistentEvent`]: ../struct.PersistedEvent.html
        #[inline]
        pub fn version(self, value: u32) -> EventBuilderWithVersion<SourceId, T> {
            EventBuilderWithVersion {
                version: value,
                event: self.event,
                source_id: self.source_id,
            }
        }

        /// Specifies the [`PersistentEvent`] sequence number and moves to the next
        /// builder state.
        ///
        /// [`PersistentEvent`]: ../struct.PersistedEvent.html
        #[inline]
        pub fn sequence_number(self, value: u32) -> EventBuilderWithSequenceNumber<SourceId, T> {
            EventBuilderWithSequenceNumber {
                sequence_number: value,
                event: self.event,
                source_id: self.source_id,
            }
        }
    }

    /// Next step in creating a new [`PersistedEvent`] carrying an Event value
    /// and its version.
    ///
    /// [`PersistentEvent`]: ../struct.PersistedEvent.html
    pub struct EventBuilderWithVersion<SourceId, T> {
        version: u32,
        event: T,
        source_id: SourceId,
    }

    impl<SourceId, T> EventBuilderWithVersion<SourceId, T> {
        /// Specifies the [`PersistentEvent`] sequence number and moves to the next
        /// builder state.
        ///
        /// [`PersistentEvent`]: ../struct.PersistedEvent.html
        #[inline]
        pub fn sequence_number(self, value: u32) -> super::PersistedEvent<SourceId, T> {
            super::PersistedEvent {
                version: self.version,
                event: self.event,
                source_id: self.source_id,
                sequence_number: value,
            }
        }
    }

    /// Next step in creating a new [`PersistedEvent`] carrying an Event value
    /// and its sequence number.
    ///
    /// [`PersistentEvent`]: ../struct.PersistedEvent.html
    pub struct EventBuilderWithSequenceNumber<SourceId, T> {
        sequence_number: u32,
        event: T,
        source_id: SourceId,
    }

    impl<SourceId, T> EventBuilderWithSequenceNumber<SourceId, T> {
        /// Specifies the [`PersistentEvent`] version and moves to the next
        /// builder state.
        ///
        /// [`PersistentEvent`]: ../struct.PersistedEvent.html
        #[inline]
        pub fn version(self, value: u32) -> super::PersistedEvent<SourceId, T> {
            super::PersistedEvent {
                version: value,
                event: self.event,
                source_id: self.source_id,
                sequence_number: self.sequence_number,
            }
        }
    }
}

/// An [`Event`] wrapper for events that have been
/// successfully committed to the [`EventStore`].
///
/// [`EventStream`]s are composed of these events.
///
/// [`Event`]: trait.EventStore.html#associatedtype.Event
/// [`EventStream`]: type.EventStream.html
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PersistedEvent<SourceId, T> {
    source_id: SourceId,
    version: u32,
    sequence_number: u32,
    #[serde(flatten)]
    event: T,
}

impl<SourceId, T> Versioned for PersistedEvent<SourceId, T> {
    #[inline]
    fn version(&self) -> u32 {
        self.version
    }
}

impl<SourceId, T> Deref for PersistedEvent<SourceId, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<SourceId, T> PersistedEvent<SourceId, T> {
    /// Creates a new [`EventBuilder`] from the provided Event value.
    ///
    /// [`EventBuilder`]: persistent/struct.EventBuilder.html
    #[inline]
    pub fn from(source_id: SourceId, event: T) -> persistent::EventBuilder<SourceId, T> {
        persistent::EventBuilder { source_id, event }
    }

    /// Returns the event sequence number.
    #[inline]
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    /// Returns the [`SourceId`] of the persisted event.
    ///
    /// [`SourceId`]: trait.EventStore.html#associatedType.SourceId
    #[inline]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
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

/// Specifies the optimistic locking level when performing [`append`] from
/// an [`EventStore`].
///
/// Check out [`append`] documentation for more info.
///
/// [`append`]: trait.EventStore.html#method.append
/// [`EventStore`]: trait.EventStore.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expected {
    /// Append events disregarding the current [`Aggregate`] version.
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    Any,

    /// Append events only if the current version of the [`Aggregate`]
    /// is the one specified by the value provided here.
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    Exact(u32),
}

/// Stream type returned by the [`EventStore::stream`] method.
///
/// [`EventStore::stream`]: trait.EventStore.html#method.stream
pub type EventStream<'a, S> = BoxStream<
    'a,
    Result<
        PersistedEvent<<S as EventStore>::SourceId, <S as EventStore>::Event>,
        <S as EventStore>::Error,
    >,
>;

/// Error type returned by [`append`] in [`EventStore`] implementations.
///
/// [`append`]: trait.EventStore.html#method.append
/// [`EventStore`]: trait.EventStore.html
pub trait AppendError: std::error::Error {
    /// Returns true if the error is due to a version conflict
    /// during [`append`].
    ///
    /// [`append`]: trait.EventStore.html#method.append
    fn is_conflict_error(&self) -> bool;
}

impl AppendError for std::convert::Infallible {
    fn is_conflict_error(&self) -> bool {
        false
    }
}

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
    type Error: AppendError;

    /// Appends a new list of [`Event`]s to the Event Store, for the Source
    /// entity specified by [`SourceId`].
    ///
    /// `append` is a transactional operation: it either appends all the events,
    /// or none at all and returns an [`AppendError`].
    ///
    /// The desired version for the new [`Event`]s to append must be specified
    /// through an [`Expected`] element.
    ///
    /// When using `Expected::Any`, no checks on the current [`Aggregate`]
    /// values will be performed, disregarding optimistic locking.
    ///
    /// When using `Expected::Exact`, the Store will check that the current
    /// version of the [`Aggregate`] is _exactly_ the one specified.
    ///
    /// If the version is not the one expected from the Store, implementations
    /// should raise an [`AppendError::Conflict`] error.
    ///
    /// Implementations could decide to return an error if the expected
    /// version is different from the one supplied in the method invocation.
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    /// [`AppendError`]: enum.AppendError.html
    /// [`AppendError::Conflict`]: enum.AppendError.html
    fn append(
        &mut self,
        source_id: Self::SourceId,
        version: Expected,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<u32, Self::Error>>;

    /// Streams a list of [`Event`]s from the `EventStore` back to the application,
    /// by specifying the desired [`SourceId`] and [`Select`] operation.
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
        source_id: Self::SourceId,
        select: Select,
    ) -> BoxFuture<Result<EventStream<Self>, Self::Error>>;

    /// Streams a list of [`Event`]s from the `EventStore` back to the application,
    /// disregarding the [`SourceId`] values but using a [`Select`] operation.
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
    fn stream_all(&self, select: Select) -> BoxFuture<Result<EventStream<Self>, Self::Error>>;

    /// Drops all the [`Event`]s related to one `Source`, specified by
    /// the provided [`SourceId`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    fn remove(&mut self, source_id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>>;
}
