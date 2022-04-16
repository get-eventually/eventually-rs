use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use futures::TryStreamExt;

use crate::{
    aggregate::{Aggregate, Context, Root},
    event,
    version::Version,
};

/// A Repository is an object that allows to load and save
/// an [Aggregate Root][Root] from and to a persistent data store.
#[async_trait]
pub trait Repository<T, R>: Send + Sync
where
    T: Aggregate,
    R: Root<T>,
{
    /// The error type that can be returned by the Repository implementation
    /// during loading or storing of an Aggregate Root.
    type Error;

    /// Loads an Aggregate Root instance from the data store,
    /// referenced by its unique identifier.
    async fn get(&self, id: &T::Id) -> Result<R, Self::Error>;

    /// Stores a new version of an Aggregate Root instance to the data store.
    async fn store(&self, root: &mut R) -> Result<(), Self::Error>;
}

/// List of possible errors that can be returned by an [`EventSourcedRepository`] method.
#[derive(Debug, thiserror::Error)]
pub enum EventSourcedRepositoryError<E, SE, AE> {
    /// This error is retured by [`EventSourcedRepository::get`] when the
    /// desired Aggregate [Root] could not be found in the data store.
    #[error("aggregate root was not found")]
    AggregateRootNotFound,

    /// This error is returned by [`EventSourcedRepository::get`] when
    /// the desired [Aggregate] returns an error while applying a Domain Event
    /// from the Event [Store][`event::Store`] during the _rehydration_ phase.
    ///
    /// This usually implies the Event Stream for the Aggregate
    /// contains corrupted or unexpected data.
    #[error("failed to rehydrate aggregate from event stream: {0}")]
    RehydrateAggregate(#[source] E),

    /// This error is returned by [`EventSourcedRepository::get`] when the
    /// [Event Store][`event::Store`] used by the Repository returns
    /// an unexpected error while streaming back the Aggregate's Event Stream.
    #[error("event store failed while streaming events: {0}")]
    StreamFromStore(#[source] SE),

    /// This error is returned by [`EventSourcedRepository::store`] when
    /// the [Event Store][`event::Store`] used by the Repository returns
    /// an error while saving the uncommitted Domain Events
    /// to the Aggregate's Event Stream.
    #[error("event store failed while appending events: {0}")]
    AppendToStore(#[source] AE),
}

/// An Event-sourced implementation of the [Repository] interface.
///
/// It uses an [Event Store][`event::Store`] instance to stream Domain Events
/// for a particular Aggregate, and append uncommitted Domain Events
/// recorded by an Aggregate Root.
#[derive(Debug, Clone)]
pub struct EventSourcedRepository<T, R, S>
where
    T: Aggregate,
    R: Root<T>,
    S: event::Store<StreamId = T::Id, Event = T::Event>,
{
    store: S,
    aggregate: PhantomData<T>,
    aggregate_root: PhantomData<R>,
}

impl<T, R, S> From<S> for EventSourcedRepository<T, R, S>
where
    T: Aggregate,
    R: Root<T>,
    S: event::Store<StreamId = T::Id, Event = T::Event>,
{
    fn from(store: S) -> Self {
        Self {
            store,
            aggregate: PhantomData,
            aggregate_root: PhantomData,
        }
    }
}

#[async_trait]
impl<T, R, S> Repository<T, R> for EventSourcedRepository<T, R, S>
where
    T: Aggregate,
    T::Id: Clone,
    T::Error: Debug,
    R: Root<T>,
    S: event::Store<StreamId = T::Id, Event = T::Event>,
{
    type Error = EventSourcedRepositoryError<T::Error, S::StreamError, S::AppendError>;

    async fn get(&self, id: &T::Id) -> Result<R, Self::Error> {
        let ctx = self
            .store
            .stream(id, event::VersionSelect::All)
            .map_ok(|persisted| persisted.event)
            .map_err(EventSourcedRepositoryError::StreamFromStore)
            .try_fold(None, |ctx: Option<Context<T>>, event| async {
                let new_ctx_result = match ctx {
                    None => Context::rehydrate_from(event),
                    Some(ctx) => ctx.apply_rehydrated_event(event),
                };

                let new_ctx =
                    new_ctx_result.map_err(EventSourcedRepositoryError::RehydrateAggregate)?;

                Ok(Some(new_ctx))
            })
            .await?;

        ctx.ok_or(EventSourcedRepositoryError::AggregateRootNotFound)
            .map(R::from)
    }

    async fn store(&self, root: &mut R) -> Result<(), Self::Error> {
        let events_to_commit = root.ctx_mut().take_uncommitted_events();
        let aggregate_id = root.aggregate_id();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let current_event_stream_version =
            root.ctx().version() - (events_to_commit.len() as Version);

        self.store
            .append(
                aggregate_id.clone(),
                event::StreamVersionExpected::MustBe(current_event_stream_version),
                events_to_commit,
            )
            .await
            .map_err(EventSourcedRepositoryError::AppendToStore)?;

        Ok(())
    }
}
