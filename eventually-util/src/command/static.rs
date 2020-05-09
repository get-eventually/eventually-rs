use futures::future::BoxFuture;

use eventually_core::aggregate::{Aggregate, EventOf, StateOf};
use eventually_core::command::{Handler as CommandHandler, Result};

pub trait Handler {
    type Command;
    type Aggregate: Aggregate;
    type Error;

    fn as_handler() -> AsHandler<Self>
    where
        Self: Sized,
    {
        AsHandler(std::marker::PhantomData)
    }

    fn handle(
        state: &StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> BoxFuture<Result<EventOf<Self::Aggregate>, Self::Error>>;
}

#[derive(Debug, Clone)]
pub struct AsHandler<T>(std::marker::PhantomData<T>);

impl<T: Handler> CommandHandler for AsHandler<T>
where
    T: Send + Sync,
    T::Command: Send + Sync,
    StateOf<T::Aggregate>: Send + Sync,
{
    type Command = T::Command;
    type Aggregate = T::Aggregate;
    type Error = T::Error;

    fn handle<'a, 'b: 'a>(
        &'a self,
        state: &'b StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<EventOf<Self::Aggregate>, Self::Error>> {
        Box::pin(T::handle(state, command))
    }
}
