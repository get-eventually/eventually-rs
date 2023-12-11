//! Module containing the definition of a [Repository], to fetch and store
//! Aggregate Roots from a data store.
//!
//! If you are looking for the Event-sourced implementation of an Aggregate Repository,
//! take a look at [EventSourced].

pub mod any;
pub mod event_sourced;

use std::fmt::Debug;

use async_trait::async_trait;

// Public re-exports.
pub use self::any::{AnyRepository, AnyRepositoryExt};
pub use self::event_sourced::EventSourced;
// Crate imports.
use crate::aggregate;
use crate::aggregate::Aggregate;

/// Error returned by a call to [Repository::get].
/// This type is used to check whether an Aggregate Root has been found or not.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetError<I> {
    /// This error is retured by [Repository::get] when the
    /// desired Aggregate [Root] could not be found in the data store.
    #[error("aggregate root was not found")]
    NotFound,

    /// Error variant returned by [Repository::get] when the underlying
    /// concrete implementation has encountered an error.
    #[error("failed to get aggregate root: {0}")]
    Inner(#[from] I),
}

/// A Repository is an object that allows to load and save
/// an [Aggregate Root][Root] from and to a persistent data store.
#[async_trait]
pub trait Repository<T>: Send + Sync
where
    T: Aggregate,
{
    /// Error type returned by the concrete implementation of the trait.
    /// It is returned in [get] using [GetError::Other].
    type GetError: Send + Sync;

    /// Error type returned by the concrete implementation of the trait.
    type SaveError: Send + Sync;

    /// Loads an Aggregate Root instance from the data store,
    /// referenced by its unique identifier.
    async fn get(&self, id: &T::Id) -> Result<aggregate::Root<T>, GetError<Self::GetError>>;

    /// Saves a new version of an Aggregate Root instance to the data store.
    async fn save(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::SaveError>;
}
