use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use futures::TryStreamExt;

use crate::{event, event::Event, version::Version};

pub trait Aggregate: Sized + Send + Sync {
    type Id: Send + Sync;
    type Event: Send + Sync;
    type Error: Send + Sync;

    fn aggregate_id(&self) -> &Self::Id;

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Context<T>
where
    T: Aggregate,
{
    aggregate: T,
    version: Version,
    recorded_events: Vec<Event<T::Event>>,
}

impl<T> Context<T>
where
    T: Aggregate,
{
    pub fn state(&self) -> &T {
        &self.aggregate
    }

    fn take_uncommitted_events(&mut self) -> Vec<Event<T::Event>> {
        std::mem::take(&mut self.recorded_events)
    }

    fn rehydrate_from(event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.payload)?,
            recorded_events: Vec::default(),
        })
    }

    fn apply_rehydrated_event(mut self, event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        self.aggregate = T::apply(Some(self.aggregate), event.payload)?;
        self.version += 1;

        Ok(self)
    }
}

impl<T> Context<T>
where
    T: Aggregate + Clone,
    T::Event: Clone,
{
    pub fn record_new(event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.payload.clone())?,
            recorded_events: vec![event],
        })
    }

    pub fn record_that(&mut self, event: Event<T::Event>) -> Result<(), T::Error> {
        self.aggregate = T::apply(Some(self.aggregate.clone()), event.payload.clone())?;
        self.recorded_events.push(event);
        self.version += 1;

        Ok(())
    }
}

pub trait Root<T>: From<Context<T>> + Send + Sync
where
    T: Aggregate,
{
    fn ctx(&self) -> &Context<T>;

    fn ctx_mut(&mut self) -> &mut Context<T>;
}

#[async_trait]
pub trait Repository<T, R>: Send + Sync
where
    T: Aggregate,
    R: Root<T>,
{
    type Error;

    async fn get(&self, id: &T::Id) -> Result<R, Self::Error>;

    async fn store(&self, root: &mut R) -> Result<(), Self::Error>;
}

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
    S::AppendError: Debug,
    S::StreamError: Debug,
{
    type Error = ();

    async fn get(&self, id: &T::Id) -> Result<R, Self::Error> {
        let ctx = self
            .store
            .stream(id, event::VersionSelect::All)
            .map_ok(|event| event.payload)
            .try_fold(None, |ctx: Option<Context<T>>, event| async {
                Ok(match ctx {
                    None => Some(Context::rehydrate_from(event).unwrap()),
                    Some(ctx) => Some(ctx.apply_rehydrated_event(event).unwrap()),
                })
            })
            .await
            .unwrap();

        match ctx {
            None => unimplemented!("return ErrNotFound!"),
            Some(ctx) => Ok(R::from(ctx)),
        }
    }

    async fn store(&self, root: &mut R) -> Result<(), Self::Error> {
        let events_to_commit = root.ctx_mut().take_uncommitted_events();
        let aggregate_id = root.ctx().state().aggregate_id();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let current_event_stream_version = root.ctx().version - (events_to_commit.len() as Version);

        let new_version = self
            .store
            .append(
                aggregate_id.clone(),
                event::StreamVersionExpected::MustBe(current_event_stream_version),
                events_to_commit,
            )
            .await
            .unwrap();

        root.ctx_mut().version = new_version;

        Ok(())
    }
}
