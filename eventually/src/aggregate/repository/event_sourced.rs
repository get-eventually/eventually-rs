//! Contains the impementation of the [EventSourced] [Repository] trait,
//! necessary to use the Event Sourcing pattern to rehydrate an [Aggregate]
//! state using an [event::Store].

use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use futures::TryStreamExt;

use crate::aggregate::{self, repository, Aggregate, Repository};
use crate::event;
use crate::version::Version;

/// List of possible errors that can be returned by [EventSourced::get].
#[derive(Debug, thiserror::Error)]
pub enum GetError<R, S> {
    /// This error is returned by [`EventSourced::get`] when
    /// the desired [Aggregate] returns an error while applying a Domain Event
    /// from the Event [Store][`event::Store`] during the _rehydration_ phase.
    ///
    /// This usually implies the Event Stream for the Aggregate
    /// contains corrupted or unexpected data.
    #[error("failed to rehydrate aggregate from event stream: {0}")]
    Rehydrate(#[source] R),

    /// This error is returned by [`EventSourced::get`] when the
    /// [Event Store][`event::Store`] used by the Repository returns
    /// an unexpected error while streaming back the Aggregate's Event Stream.
    #[error("event store failed while streaming events: {0}")]
    Stream(#[source] S),
}

/// List of possible errors that can be returned by [EventSourced::save].
#[derive(Debug, thiserror::Error)]
pub enum SaveError<T> {
    /// This error is returned by [EventSourced::save] when
    /// the [event::Store] used by the Repository returns
    /// an error while saving the uncommitted Domain Events
    /// to the Aggregate's Event Stream.
    #[error("event store failed while appending events: {0}")]
    Append(#[from] T),
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
impl<T, S> Repository<T> for EventSourced<T, S>
where
    T: Aggregate,
    T::Id: Clone,
    T::Error: Debug,
    S: event::Store<T::Id, T::Event>,
{
    type GetError = GetError<T::Error, <S as event::Streamer<T::Id, T::Event>>::Error>;
    type SaveError = SaveError<<S as event::Appender<T::Id, T::Event>>::Error>;

    async fn get(
        &self,
        id: &T::Id,
    ) -> Result<aggregate::Root<T>, repository::GetError<Self::GetError>> {
        let ctx = self
            .store
            .stream(id, event::VersionSelect::All)
            .map_ok(|persisted| persisted.event)
            .map_err(GetError::Stream)
            .try_fold(None, |ctx: Option<aggregate::Root<T>>, event| async {
                let new_ctx_result = match ctx {
                    None => aggregate::Root::<T>::rehydrate_from(event),
                    Some(ctx) => ctx.apply_rehydrated_event(event),
                };

                let new_ctx = new_ctx_result.map_err(GetError::Rehydrate)?;

                Ok(Some(new_ctx))
            })
            .await?;

        ctx.ok_or(repository::GetError::NotFound)
    }

    async fn save(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::SaveError> {
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
            .map_err(SaveError::Append)?;

        Ok(())
    }
}
