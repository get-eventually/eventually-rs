//! Module containing support for the Event Store.
//!
//! ## Event Store and `eventually`
//!
//! An Event Store is an ordered, append-only log of many Domain Events
//! related to one or more Aggregate instances.
//!
//! The Domain Events committed to the Event Store can be streamed back into
//! the application to load the latest value of an Aggregate state,
//! by using the concept of Event Streams.
//!
//! Since Domain Events in the Store are ordered both globally and
//! on an Aggregate-instance basis, the Event Stream can either be global
//! or related to a single Aggregate.
//!
//! `eventually` adds support for the Event Store through the [`EventStore`]
//! trait.
//!
//! You rarely need to use the [`EventStore`] directly when writing your
//! application, since `eventually` exposes multiple utilities that
//! instrument the usage of the store for you. For example:
//! * [`Repository`] for retrieving, saving and deleting Aggregates
//! * [`Projector`] to run [`Projection`]s (check out the [`projection`
//!   module documentation])
//!
//! [`EventStore`]: trait.EventStore.html
//! [`Repository`]: ../repository/struct.Repository.html
//! [`Projector`]: ../struct.Projector.html
//! [`Projection`]: ../trait.Projection.html
//! [`projection` module documentation]: ../projection/index.html

use std::ops::Deref;

use futures::future::BoxFuture;
use futures::stream::BoxStream;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::versioning::Versioned;

/// Selection operation for the events to capture in an [`EventStream`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Select {
    /// To return all the [`Event`](EventStore::Event)s in the [`EventStream`].
    All,

    /// To return a slice of the [`EventStream`], starting from
    /// those [`Event`](EventStore::Event)s with version **greater or equal**
    /// than the one specified in this variant.
    From(u32),
}

/// Specifies the optimistic locking level when performing
/// [`append`](EventStore::append) from an [`EventStore`].
///
/// Check out [`append`](EventStore::append) documentation for more info.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expected {
    /// Append events disregarding the current
    /// [`Aggregate`](super::aggregate::Aggregate) version.
    Any,

    /// Append events only if the current version of the
    /// [`Aggregate`](super::aggregate::Aggregate) is the one specified by
    /// the value provided here.
    Exact(u32),
}

/// Stream type returned by the [`EventStore::stream`] method.
pub type EventStream<'a, S> = BoxStream<
    'a,
    Result<
        Persisted<<S as EventStore>::SourceId, <S as EventStore>::Event>,
        <S as EventStore>::Error,
    >,
>;

/// Error type returned by [`append`](EventStore::append) in [`EventStore`]
/// implementations.
pub trait AppendError: std::error::Error {
    /// Returns true if the error is due to a version conflict
    /// during [`append`](EventStore::append).
    fn is_conflict_error(&self) -> bool;
}

impl AppendError for std::convert::Infallible {
    fn is_conflict_error(&self) -> bool {
        false
    }
}

/// An Event Store is an append-only, ordered list of
/// [`Event`](super::aggregate::Aggregate::Event)s for a certain "source" --
/// e.g. an [`Aggregate`](super::aggregate::Aggregate).
pub trait EventStore {
    /// Type of the Source id, typically an
    /// [`AggregateId`](super::aggregate::AggregateId).
    type SourceId: Eq;

    /// Event to be stored in the [`EventStore`], typically an
    /// [`Aggregate::Event`](super::aggregate::Aggregate::Event).
    type Event;

    /// Possible errors returned by the [`EventStore`] when requesting
    /// operations.
    type Error: AppendError;

    /// Appends a new list of [`Event`](EventStore::Event)s to the Event Store,
    /// for the Source entity specified by
    /// [`SourceId`](EventStore::SourceId).
    ///
    /// `append` is a transactional operation: it either appends all the events,
    /// or none at all and returns an [`AppendError`].
    ///
    /// The desired version for the new [`Event`](EventStore::Event)s to append
    /// must be specified through an [`Expected`] element.
    ///
    /// When using [`Expected::Any`], no checks on the current
    /// [`Aggregate`](crate::aggregate::Aggregate) values will be performed,
    /// disregarding optimistic locking.
    ///
    /// When using [`Expected::Exact`], the Store will check that the current
    /// version of the [`Aggregate`](crate::aggregate::Aggregate) is _exactly_
    /// the one specified.
    ///
    /// If the version is not the one expected from the Store, implementations
    /// should raise a conflict error.
    ///
    /// Implementations could decide to return an error if the expected
    /// version is different from the one supplied in the method invocation.
    fn append(
        &mut self,
        source_id: Self::SourceId,
        version: Expected,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<u32, Self::Error>>;

    /// Streams a list of [`Event`](EventStore::Event)s from the [`EventStore`]
    /// back to the application, by specifying the desired
    /// [`SourceId`](EventStore::SourceId) and [`Select`] operation.
    ///
    /// [`SourceId`](EventStore::SourceId) will be used to request a particular
    /// `EventStream`.
    ///
    /// [`Select`] specifies the selection strategy for the
    /// [`Event`](EventStore::Event)s in the returned [`EventStream`]: take
    /// a look at type documentation for all the available options.
    fn stream(
        &self,
        source_id: Self::SourceId,
        select: Select,
    ) -> BoxFuture<Result<EventStream<Self>, Self::Error>>;

    /// Streams a list of [`Event`](EventStore::Event)s from the [`EventStore`]
    /// back to the application, disregarding the
    /// [`SourceId`](EventStore::SourceId) values but using a [`Select`]
    /// operation.
    ///
    /// [`SourceId`](EventStore::SourceId) will be used to request a particular
    /// [`EventStream`].
    ///
    /// [`Select`] specifies the selection strategy for the
    /// [`Event`](EventStore::Event)s in the returned [`EventStream`]: take
    /// a look at type documentation for all the available options.
    fn stream_all(&self, select: Select) -> BoxFuture<Result<EventStream<Self>, Self::Error>>;

    /// Drops all the [`Event`](EventStore::Event)s related to one `Source`,
    /// specified by the provided [`SourceId`](EventStore::SourceId).
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    fn remove(&mut self, source_id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>>;
}

/// An [`Event`](EventStore::Event) wrapper for events that have been
/// successfully committed to the [`EventStore`].
///
/// [`EventStream`]s are composed of these events.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Persisted<SourceId, T> {
    source_id: SourceId,
    version: u32,
    sequence_number: u32,
    #[cfg_attr(feature = "serde", serde(flatten))]
    event: T,
}

impl<SourceId, T> Versioned for Persisted<SourceId, T> {
    #[inline]
    fn version(&self) -> u32 {
        self.version
    }
}

impl<SourceId, T> Deref for Persisted<SourceId, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<SourceId, T> Persisted<SourceId, T> {
    /// Creates a new [`EventBuilder`](persistent::EventBuilder) from the
    /// provided Event value.
    #[inline]
    pub fn from(source_id: SourceId, event: T) -> persistent::EventBuilder<SourceId, T> {
        persistent::EventBuilder { event, source_id }
    }

    /// Returns the event sequence number.
    #[inline]
    pub fn sequence_number(&self) -> u32 {
        self.sequence_number
    }

    /// Returns the [`SourceId`](EventStore::SourceId) of the persisted event.
    #[inline]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Unwraps the inner [`Event`](EventStore::Event) from the `Persisted`
    /// wrapper.
    #[inline]
    pub fn take(self) -> T {
        self.event
    }
}

/// Contains a type-state builder for [`Persisted`] type.
pub mod persistent {
    /// Creates a new [`Persisted`](super::Persisted) by wrapping an Event
    /// value.
    pub struct EventBuilder<SourceId, T> {
        pub(super) event: T,
        pub(super) source_id: SourceId,
    }

    impl<SourceId, T> From<(SourceId, T)> for EventBuilder<SourceId, T> {
        #[inline]
        fn from(value: (SourceId, T)) -> Self {
            let (source_id, event) = value;
            Self { event, source_id }
        }
    }

    impl<SourceId, T> EventBuilder<SourceId, T> {
        /// Specifies the [`Persisted`](super::Persisted) version and moves to
        /// the next builder state.
        #[inline]
        pub fn version(self, value: u32) -> EventBuilderWithVersion<SourceId, T> {
            EventBuilderWithVersion {
                version: value,
                event: self.event,
                source_id: self.source_id,
            }
        }

        /// Specifies the [`Persisted`](super::Persisted) sequence number and
        /// moves to the next builder state.
        #[inline]
        pub fn sequence_number(self, value: u32) -> EventBuilderWithSequenceNumber<SourceId, T> {
            EventBuilderWithSequenceNumber {
                sequence_number: value,
                event: self.event,
                source_id: self.source_id,
            }
        }
    }

    /// Next step in creating a new [`Persisted`](super::Persisted) carrying an
    /// Event value and its version.
    pub struct EventBuilderWithVersion<SourceId, T> {
        version: u32,
        event: T,
        source_id: SourceId,
    }

    impl<SourceId, T> EventBuilderWithVersion<SourceId, T> {
        /// Specifies the [`Persisted`](super::Persisted) sequence number and
        /// moves to the next builder state.
        #[inline]
        pub fn sequence_number(self, value: u32) -> super::Persisted<SourceId, T> {
            super::Persisted {
                version: self.version,
                event: self.event,
                source_id: self.source_id,
                sequence_number: value,
            }
        }
    }

    /// Next step in creating a new [`Persisted`](super::Persisted) carrying an
    /// Event value and its sequence number.
    pub struct EventBuilderWithSequenceNumber<SourceId, T> {
        sequence_number: u32,
        event: T,
        source_id: SourceId,
    }

    impl<SourceId, T> EventBuilderWithSequenceNumber<SourceId, T> {
        /// Specifies the [`Persisted`](super::Persisted) version and moves to
        /// the next builder state.
        #[inline]
        pub fn version(self, value: u32) -> super::Persisted<SourceId, T> {
            super::Persisted {
                version: value,
                event: self.event,
                source_id: self.source_id,
                sequence_number: self.sequence_number,
            }
        }
    }
}
