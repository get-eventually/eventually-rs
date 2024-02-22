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
use crate::{event, version};

/// An Event-sourced implementation of the [Repository] interface.
///
/// It uses an [Event Store][event::Store] instance to stream Domain Events
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
                version::Check::MustBe(current_event_stream_version),
                events_to_commit,
            )
            .await?;

        Ok(())
    }
}
