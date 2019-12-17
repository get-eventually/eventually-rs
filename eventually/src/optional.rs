use std::future::Future;

use crate::{aggregate, Aggregate, CommandHandler};

pub type StateOf<A: OptionalAggregate> = A::State;
pub type EventOf<A: OptionalAggregate> = A::Event;

pub trait Handler {
    type Command;
    type Aggregate: OptionalAggregate;
    type Error;
    type Result: Future<Output = Result<Vec<EventOf<Self::Aggregate>>, Self::Error>>;

    fn handle_initial(&self, command: Self::Command) -> Self::Result;

    fn handle_next(&self, state: &StateOf<Self::Aggregate>, command: Self::Command)
        -> Self::Result;

    fn as_handler(self) -> AsHandler<Self>
    where
        Self: Sized,
    {
        AsHandler(self)
    }
}

pub struct AsHandler<H>(H);

impl<H> CommandHandler for AsHandler<H>
where
    H: Handler,
{
    type Command = H::Command;
    type Aggregate = AsAggregate<H::Aggregate>;
    type Error = H::Error;
    type Result = H::Result;

    fn handle(
        &self,
        state: &aggregate::StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> Self::Result {
        match state {
            None => self.0.handle_initial(command),
            Some(state) => self.0.handle_next(state, command),
        }
    }
}

pub trait OptionalAggregate {
    type State;
    type Event;
    type Error;

    fn initial(event: Self::Event) -> Result<Self::State, Self::Error>;

    fn apply_next(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;
}

pub struct AsAggregate<T>(std::marker::PhantomData<T>);

impl<T: OptionalAggregate> Aggregate for AsAggregate<T> {
    type State = Option<T::State>;
    type Event = T::Event;
    type Error = T::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        Ok(Some(match state {
            None => T::initial(event)?,
            Some(state) => T::apply_next(state, event)?,
        }))
    }
}
