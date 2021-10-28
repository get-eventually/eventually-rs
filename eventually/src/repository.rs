//! Module containing Repository implementation to retrieve,
//! save and delete Aggregates.
//!
//! ## Repository and Aggregates
//!
//! As described in the [Interacting with Aggregates using `AggregateRoot`]
//! section in the `aggregate` module-level documentation, in order to
//! interact with an Aggregate instance you need to use an
//! [`AggregateRoot`].
//!
//! To get an [`AggregateRoot`], you can use a [`Repository`] instance.
//!
//! The [`Repository`] allows to **retrieve**, **save** and **remove**
//! specific Aggregate instances, by using an underlying [`EventStore`]
//! implementation that handles the Aggregate's events.
//!
//! A [`Repository`] will **always** return an [`AggregateRoot`] instance
//! on read, whether or not events are present in the [`EventStore`].
//!
//! Use the [`Repository`] to implement your bounded-context application
//! logic, for example in HTTP or RPC handlers.
//!
//! [Interacting with Aggregates using `AggregateRoot`]:
//! ../aggregate/index.html#interacting-with-aggregates-using-aggregateroot
//! [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
//! [`Repository`]: struct.Repository.html
//! [`EventStore`]: ../store/trait.EventStore.html

use std::fmt::Debug;

use futures::stream::TryStreamExt;

use crate::aggregate::{Aggregate, AggregateRoot, AggregateRootFactory};
use crate::store::{EventStore, Expected, Select};
use crate::versioning::Versioned;

/// Error type returned by the [`Repository`].
#[derive(Debug, thiserror::Error)]
pub enum Error<A, S>
where
    A: std::error::Error + 'static,
    S: std::error::Error + 'static,
{
    /// Error returned by the [`Aggregate`], usually when recreating the
    /// [`State`](Aggregate::State).
    #[error("failed to rebuild aggregate state: {0}")]
    Aggregate(#[source] A),

    /// Error returned by the underlying [`EventStore`].
    #[error("event store failed: {0}")]
    Store(#[source] S),
}

/// Result type returned by the [`Repository`].
pub type Result<T, A, S> =
    std::result::Result<T, Error<<A as Aggregate>::Error, <S as EventStore>::Error>>;

/// Implementation of the [Repository pattern] for storing, retrieving
/// and deleting [`Aggregate`]s.
///
/// A `Repository` instruments an [`EventStore`] to:
///
/// * **Insert** [`Event`]s in the [`EventStore`] for an Aggregate, using the
///   [`AggregateRoot`],
/// * **Get** all the [`Event`]s in the [`EventStore`] and rebuild the [`State`]
///   of an Aggregate,
/// into a new [`AggregateRoot`] instance,
/// * **Remove** all the [`Event`]s for an Aggregate in the [`EventStore`].
///
/// [Repository pattern]: https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/infrastructure-persistence-layer-design#the-repository-pattern
/// [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
/// [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
/// [`EventStore`]: ../store/trait.EventStore.html
#[derive(Clone)]
pub struct Repository<T, Store>
where
    T: Aggregate + Clone + 'static,
    Store: EventStore<SourceId = T::Id, Event = T::Event>,
{
    factory: AggregateRootFactory<T>,
    store: Store,
}

impl<T, Store> Repository<T, Store>
where
    T: Aggregate + Clone,
    Store: EventStore<SourceId = T::Id, Event = T::Event>,
{
    /// Creates a new `Repository` instance, using the [`Aggregate`]
    /// and [`EventStore`] provided.
    ///
    /// [`EventStore`]: ../store/trait.EventStore.html
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    #[inline]
    pub fn new(factory: AggregateRootFactory<T>, store: Store) -> Self {
        Repository { factory, store }
    }
}

impl<T, Store> Repository<T, Store>
where
    T: Aggregate + Clone,
    T::Id: Debug + Clone,
    T::Event: Clone,
    T::Error: std::error::Error + 'static,
    Store: EventStore<SourceId = T::Id, Event = T::Event>,
    Store::Error: std::error::Error + 'static,
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
    #[cfg_attr(
        feature = "with-tracing",
        tracing::instrument(level = "info", name = "Repository::get", skip(self))
    )]
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
            .map(|(version, state)| self.factory.build_with_state(id, version, state))
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
    #[cfg_attr(
        feature = "with-tracing",
        tracing::instrument(level = "info", name = "Repository::add", skip(self, root))
    )]
    pub async fn add(&mut self, mut root: AggregateRoot<T>) -> Result<AggregateRoot<T>, T, Store> {
        let mut version = root.version();
        let events = root.take_events_to_commit();

        if events.is_empty() {
            return Ok(root);
        }
        version = self
            .store
            .append(root.id().clone(), Expected::Exact(version), events)
            .await
            .map_err(Error::Store)?;

        Ok(root.with_version(version))
    }

    /// Removes the specified [`Aggregate`] from the `Repository`,
    /// using the provided [`AggregateId`].
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    #[cfg_attr(
        feature = "with-tracing",
        tracing::instrument(level = "info", name = "Repository::remove", skip(self))
    )]
    pub async fn remove(&mut self, id: T::Id) -> Result<(), T, Store> {
        self.store.remove(id).await.map_err(Error::Store)
    }
}
