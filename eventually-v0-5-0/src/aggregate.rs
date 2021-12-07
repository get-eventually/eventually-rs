use std::marker::PhantomData;

use async_trait::async_trait;

use crate::{
    event,
    event::{Event, Events},
};

pub trait Aggregate: Sized + Send + Sync {
    type Id;
    type Event;
    type Error;

    fn aggregate_id(&self) -> &Self::Id;

    fn apply(state: Option<&mut Self>, event: Self::Event) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
#[must_use]
pub struct Context<T>
where
    T: Aggregate,
{
    aggregate: Option<T>,
    recorded_events: Events<T::Event>,
}

impl<T> Default for Context<T>
where
    T: Aggregate,
{
    fn default() -> Self {
        Self {
            aggregate: None,
            recorded_events: Vec::new(),
        }
    }
}

impl<T> Context<T>
where
    T: Aggregate,
    T::Event: Clone,
{
    pub fn state(&self) -> Option<&T> {
        self.aggregate.as_ref()
    }

    pub fn record_that(&mut self, event: Event<T::Event>) -> Result<(), T::Error> {
        T::apply(self.aggregate.as_mut(), event.clone().payload)?;

        self.recorded_events.push(event);

        Ok(())
    }
}

pub trait Root<T>: From<Context<T>>
where
    T: Aggregate,
{
    fn ctx(&self) -> &Context<T>;

    fn ctx_mut(&mut self) -> &mut Context<T>;
}

#[async_trait]
pub trait Repository<T, R>
where
    T: Aggregate,
    R: Root<T>,
{
    type Error;

    async fn get(id: &T::Id) -> Result<T, Self::Error>;

    async fn store(&mut root: R) -> Result<(), Self::Error>;
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

// impl<T, R, S> Repository<T, R> for EventSourcedRepository<T, R, S>
// where
//     T: Aggregate,
//     R: Root<T>,
//     S: event::Store<StreamId = T::Id, Event = T::Event>,
// {
//     type Error =
// }
