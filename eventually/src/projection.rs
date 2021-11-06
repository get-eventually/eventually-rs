//! Contain support for [`Projection`], an optimized read model
//! of an [`Aggregate`] or of a number of `Aggregate`s.
//!
//! More information about projections can be found here:
//! <https://eventstore.com/docs/getting-started/projections/index.html>
//!
//! [`Projection`]: trait.Projection.html
//! [`Aggregate`]: ../aggregate/trait.Aggregate.html

use async_trait::async_trait;

use crate::store::Persisted;

/// A `Projection` is an optimized read model (or materialized view)
/// of an [`Aggregate`] model(s), that can be assembled by left-folding
/// its previous state and a number of ordered, consecutive events.
///
/// The events passed to a `Projection` have been persisted onto
/// an [`EventStore`] first.
///
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`EventStore`]: ../store/trait.EventStore.html
#[async_trait]
pub trait Projection {
    /// Type of the Source id, typically an [`AggregateId`].
    ///
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    type SourceId: Eq;

    /// Event to be stored in the `EventStore`, typically an
    /// [`Aggregate::Event`].
    ///
    /// [`Aggregate::Event`]:
    /// ../aggregate/trait.Aggregate.html#associatedtype.Event
    type Event;

    /// Type of the possible error that might occur when projecting
    /// the next state.
    type Error;

    /// Updates the next value of the `Projection` using the provided event
    /// value.
    async fn project(
        &mut self,
        event: Persisted<Self::SourceId, Self::Event>,
    ) -> Result<(), Self::Error>;
}
