//! Contains the [Repository pattern] implementation for [`AggregateRoot`]
//! instances.
//!
//! Check out [`Repository`] for more information.
//!
//! [`Repository`]: struct.Repository.html
//! [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
//! [Repository pattern]: https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/infrastructure-persistence-layer-design#the-repository-pattern

use std::error::Error as StdError;
use std::fmt::Debug;

use futures::stream::TryStreamExt;

use thiserror::Error as ThisError;

use crate::aggregate::{Aggregate, AggregateRoot, AggregateRootBuilder};
use crate::store::{EventStore, Select};

/// Error type returned by the [`Repository`].
///
/// [`Repository`]: trait.Repository.html
#[derive(Debug, ThisError, PartialEq, Eq)]
pub enum Error<A, S>
where
    A: StdError + 'static,
    S: StdError + 'static,
{
    /// Error returned by the [`Aggregate`], usually when recreating the [`State`].
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
    #[error("failed to rebuild aggregate state: {0}")]
    Aggregate(#[source] A),

    /// Error returned by the underlying [`EventStore`].
    ///
    /// [`EventStore`]: ../store/trait.EventStore.html
    #[error("event store failed: {0}")]
    Store(#[source] S),
}

/// Result type returned by the [`Repository`].
///
/// [`Repository`]: ../struct.Repository.html
pub type Result<T, A, S> =
    std::result::Result<T, Error<<A as Aggregate>::Error, <S as EventStore>::Error>>;

/// Implementation of the [Repository pattern] for storing, retrieving
/// and deleting [`Aggregate`]s.
///
/// A `Repository` instruments an [`EventStore`] to:
///
/// * **Insert** [`Event`]s in the [`EventStore`] for an Aggregate, using the [`AggregateRoot`],
/// * **Get** all the [`Event`]s in the [`EventStore`] and rebuild the [`State`] of an Aggregate,
/// into a new [`AggregateRoot`] instance,
/// * **Remove** all the [`Event`]s for an Aggregate in the [`EventStore`].
///
/// [Repository pattern]: https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/infrastructure-persistence-layer-design#the-repository-pattern
/// [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
/// [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
/// [`EventStore`]: ../store/trait.EventStore.html
pub struct Repository<T, Store>
where
    T: Aggregate + 'static,
{
    builder: AggregateRootBuilder<T>,
    store: Store,
}

impl<T, Store> Repository<T, Store>
where
    T: Aggregate,
    Store: EventStore<SourceId = T::Id, Event = T::Event>,
{
    /// Creates a new `Repository` instance, using the [`Aggregate`]
    /// and [`EventStore`] provided.
    ///
    /// [`EventStore`]: ../store/trait.EventStore.html
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    #[inline]
    pub fn new(builder: AggregateRootBuilder<T>, store: Store) -> Self {
        Repository { builder, store }
    }
}

impl<T, Store> Repository<T, Store>
where
    T: Aggregate + Debug + Clone,
    T::Id: Clone,
    T::Event: Clone,
    T::Error: StdError + 'static,
    Store: EventStore<SourceId = T::Id, Event = T::Event> + Debug,
    Store::Error: StdError + 'static,
{
    /// Returns the [`Aggregate`] from the `Repository` with the specified id,
    /// if any.
    ///
    /// In case the [`Aggregate`] with the specified id exists, returns
    /// a new [`AggregateRoot`] instance with its latest [`State`].
    ///
    /// Otherwise, `None` is returned.
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
    /// [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
    pub async fn get(&self, id: T::Id) -> Result<AggregateRoot<T>, T, Store> {
        self.store
            .stream(id.clone(), Select::All)
            .await
            .map_err(Error::Store)?
            // Re-map any errors from the Stream into a Repository error
            .map_err(Error::Store)
            // Try to fold all the Events into an Aggregate State.
            .try_fold(
                (0, T::State::default()),
                |(version, state), event| async move {
                    // Always consider the max version number for the next version.
                    let new_version = std::cmp::max(event.version(), version);
                    let state = T::apply(state, event.take()).map_err(Error::Aggregate)?;

                    Ok((new_version, state))
                },
            )
            .await
            // ...and map the State to a new AggregateRoot only if there is
            // at least one Event coming from the Event Stream.
            .map(|(version, state)| self.builder.build_with_state(id, version, state))
    }

    /// Adds a new [`State`] of the [`Aggregate`] into the `Repository`,
    /// through an [`AggregateRoot`] instance.
    ///
    /// Returns [`Error::NoEvents`] if there are no [`Event`]s to commit
    /// in the [`AggregateRoot`].
    ///
    /// [`Error::NoEvents`]: ../repository/enum.Error.html#variant.NoEvents
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
    /// [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
    /// [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
    pub async fn add(&mut self, mut root: AggregateRoot<T>) -> Result<AggregateRoot<T>, T, Store> {
        let mut version = root.version();
        let events_to_commit = root.take_events_to_commit();

        if let Some(events) = events_to_commit {
            if !events.is_empty() {
                // Version is incremented at each events flush.
                version += 1;

                self.store
                    .append(root.id().clone(), version, events)
                    .await
                    .map_err(Error::Store)?;
            }
        }

        Ok(root.with_version(version))
    }

    /// Removes the specified [`Aggregate`] from the `Repository`,
    /// using the provided [`AggregateId`].
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    pub async fn remove(&mut self, id: T::Id) -> Result<(), T, Store> {
        self.store.remove(id).await.map_err(Error::Store)
    }
}
