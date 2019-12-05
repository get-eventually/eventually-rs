use std::future::Future;

use crate::{
    aggregate::{Aggregate, EventOf, StateOf},
    command::Handler,
};

pub trait StaticHandler {
    type Command;
    type Aggregate: Aggregate;
    type Error;
    type Result: Future<Output = Result<Vec<EventOf<Self::Aggregate>>, Self::Error>>;

    fn as_handler() -> AsHandler<Self>
    where
        Self: Sized,
    {
        AsHandler(std::marker::PhantomData)
    }

    fn handle(state: &StateOf<Self::Aggregate>, command: Self::Command) -> Self::Result;
}

pub struct AsHandler<T>(std::marker::PhantomData<T>);

impl<T: StaticHandler> Handler for AsHandler<T>
where
    T: Send + Sync,
    T::Command: Send + Sync,
    StateOf<T::Aggregate>: Send + Sync,
{
    type Command = T::Command;
    type Aggregate = T::Aggregate;
    type Error = T::Error;
    type Result = T::Result;

    fn handle(&self, state: &StateOf<Self::Aggregate>, command: Self::Command) -> Self::Result {
        T::handle(state, command)
    }
}
