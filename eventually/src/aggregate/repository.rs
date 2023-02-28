use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use futures::TryStreamExt;

use crate::{aggregate, aggregate::Aggregate, event, version::Version};

/// Error returned by a call to [`Repository::get`].
/// This type is used to check whether an Aggregate Root has been found or not.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetError<I> {
    /// This error is retured by [`Repository::get`] when the
    /// desired Aggregate [Root] could not be found in the data store.
    #[error("aggregate root was not found")]
    NotFound,
    #[error(transparent)]
    Inner(#[from] I),
}

#[async_trait]
pub trait Getter<T>: Send + Sync
where
    T: Aggregate,
{
    type Error: Send + Sync;

    /// Loads an Aggregate Root instance from the data store,
    /// referenced by its unique identifier.
    async fn get(&self, id: &T::Id) -> Result<aggregate::Root<T>, GetError<Self::Error>>;
}

#[async_trait]
pub trait Saver<T>: Send + Sync
where
    T: Aggregate,
{
    type Error: Send + Sync;

    /// Saves a new version of an Aggregate Root instance to the data store.
    async fn save(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::Error>;
}

/// A Repository is an object that allows to load and save
/// an [Aggregate Root][Root] from and to a persistent data store.
pub trait Repository<T>: Getter<T> + Saver<T> + Send + Sync
where
    T: Aggregate,
{
}

impl<T, R> Repository<T> for R
where
    T: Aggregate,
    R: Getter<T> + Saver<T> + Send + Sync,
{
}

/// List of possible errors that can be returned by an [`EventSourced`] method.
#[derive(Debug, thiserror::Error)]
pub enum EventSourcedError<E, SE, AE> {
    /// This error is returned by [`EventSourced::get`] when
    /// the desired [Aggregate] returns an error while applying a Domain Event
    /// from the Event [Store][`event::Store`] during the _rehydration_ phase.
    ///
    /// This usually implies the Event Stream for the Aggregate
    /// contains corrupted or unexpected data.
    #[error("failed to rehydrate aggregate from event stream: {0}")]
    RehydrateAggregate(#[source] E),

    /// This error is returned by [`EventSourced::get`] when the
    /// [Event Store][`event::Store`] used by the Repository returns
    /// an unexpected error while streaming back the Aggregate's Event Stream.
    #[error("event store failed while streaming events: {0}")]
    StreamFromStore(#[source] SE),

    /// This error is returned by [`EventSourced::store`] when
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
pub struct EventSourced<T, S>
where
    T: Aggregate,
    S: event::Store<T::Id, T::Event>,
{
    store: S,
    aggregate: PhantomData<T>,
}

impl<T, S> From<S> for EventSourced<T, S>
where
    T: Aggregate,
    S: event::Store<T::Id, T::Event>,
{
    fn from(store: S) -> Self {
        Self {
            store,
            aggregate: PhantomData,
        }
    }
}

#[async_trait]
impl<T, S> Getter<T> for EventSourced<T, S>
where
    T: Aggregate,
    T::Id: Clone,
    T::Error: Debug,
    S: event::Store<T::Id, T::Event>,
{
    type Error = EventSourcedError<
        T::Error,
        <S as event::Streamer<T::Id, T::Event>>::Error,
        <S as event::Appender<T::Id, T::Event>>::Error,
    >;

    async fn get(&self, id: &T::Id) -> Result<aggregate::Root<T>, GetError<Self::Error>> {
        let ctx = self
            .store
            .stream(id, event::VersionSelect::All)
            .map_ok(|persisted| persisted.event)
            .map_err(EventSourcedError::StreamFromStore)
            .try_fold(None, |ctx: Option<aggregate::Root<T>>, event| async {
                let new_ctx_result = match ctx {
                    None => aggregate::Root::<T>::rehydrate_from(event),
                    Some(ctx) => ctx.apply_rehydrated_event(event),
                };

                let new_ctx = new_ctx_result.map_err(EventSourcedError::RehydrateAggregate)?;

                Ok(Some(new_ctx))
            })
            .await?;

        ctx.ok_or(GetError::NotFound)
    }
}

#[async_trait]
impl<T, S> Saver<T> for EventSourced<T, S>
where
    T: Aggregate,
    T::Id: Clone,
    T::Error: Debug,
    S: event::Store<T::Id, T::Event>,
{
    type Error = EventSourcedError<
        T::Error,
        <S as event::Streamer<T::Id, T::Event>>::Error,
        <S as event::Appender<T::Id, T::Event>>::Error,
    >;

    async fn save(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::Error> {
        let events_to_commit = root.take_uncommitted_events();
        let aggregate_id = root.aggregate_id();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let current_event_stream_version = root.version() - (events_to_commit.len() as Version);

        self.store
            .append(
                aggregate_id.clone(),
                event::StreamVersionExpected::MustBe(current_event_stream_version),
                events_to_commit,
            )
            .await
            .map_err(EventSourcedError::AppendToStore)?;

        Ok(())
    }
}
