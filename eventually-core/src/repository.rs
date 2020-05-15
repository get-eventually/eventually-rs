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

use futures::future;
use futures::stream::TryStreamExt;

use thiserror::Error as ThisError;

use crate::aggregate::{Aggregate, AggregateId, AggregateRoot, Identifiable};
use crate::store::EventStore;

/// Error type returned by the [`Repository`].
///
/// [`Repository`]: trait.Repository.html
#[derive(Debug, ThisError, PartialEq, Eq)]
pub enum Error<A, S>
where
    A: Aggregate + Debug,
    S: EventStore + Debug,
    A::Error: StdError + 'static,
    S::Error: StdError + 'static,
{
    /// Error returned by the [`Aggregate`], usually when recreating the [`State`].
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
    #[error("failed to rebuild aggregate state: {0}")]
    Aggregate(#[source] A::Error),

    /// Error returned by the underlying [`EventStore`].
    ///
    /// [`EventStore`]: ../store/trait.EventStore.html
    #[error("event store failed: {0}")]
    Store(#[source] S::Error),

    /// Error returned by [`add`] method when trying to add an [`AggregateRoot`]
    /// that has no [`Event`]s to be flushed to the [`EventStore`].
    ///
    /// [`add`]: struct.Repository.html#method.add
    /// [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
    /// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
    /// [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
    /// [`EventStore`]: ../store/trait.EventStore.html
    #[error("no events to commit")]
    NoEvents,
}

/// Result type returned by the [`Repository`].
///
/// [`Repository`]: ../struct.Repository.html
pub type Result<T, A, S> = std::result::Result<T, Error<A, S>>;

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
    aggregate: T,
    store: Store,
}

impl<T, Store> Repository<T, Store>
where
    T: Aggregate,
    Store: EventStore<SourceId = AggregateId<T>, Event = T::Event>,
{
    /// Creates a new `Repository` instance, using the [`Aggregate`]
    /// and [`EventStore`] provided.
    ///
    /// [`EventStore`]: ../store/trait.EventStore.html
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    #[inline]
    pub fn new(aggregate: T, store: Store) -> Self {
        Repository { aggregate, store }
    }
}

impl<T, Store> Repository<T, Store>
where
    T: Aggregate + Debug + Clone,
    T::Event: Clone,
    T::State: Default + Identifiable,
    T::Error: StdError + 'static,
    AggregateId<T>: Default,
    Store: EventStore<SourceId = AggregateId<T>, Event = T::Event> + Debug,
    Store::Offset: Default,
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
    pub async fn get(&self, id: AggregateId<T>) -> Result<Option<AggregateRoot<T>>, T, Store> {
        Ok(self
            .store
            .stream(id, Store::Offset::default())
            .await
            .map_err(Error::Store)?
            // Re-map any errors from the Stream into a Repository error
            .map_err(Error::Store)
            // Try to fold all the Events into an Aggregate State.
            //
            // However, since the Event Store might not have any Events yet,
            // use an Option<T::State> with the initial value set to None...
            .try_fold(None, |state, event| {
                let state = state.unwrap_or_else(T::State::default);
                future::ready(T::apply(state, event).map(Some).map_err(Error::Aggregate))
            })
            .await?
            // ...and map the State to a new AggregateRoot only if there is
            // at least one Event coming from the Event Stream.
            .map(|state| AggregateRoot::new(self.aggregate.clone(), state)))
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
        if root.to_commit.is_none() {
            return Err(Error::NoEvents);
        }

        self.store
            .append(root.id(), root.to_commit.unwrap())
            .await
            .map_err(Error::Store)?;

        root.to_commit = None;

        Ok(root)
    }

    /// Removes the specified [`Aggregate`] from the `Repository`,
    /// using the provided [`AggregateId`].
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    pub async fn remove(&mut self, id: AggregateId<T>) -> Result<(), T, Store> {
        self.store.remove(id).await.map_err(Error::Store)
    }
}
