use async_trait::async_trait;

use crate::{
    aggregate::Aggregate,
    command::{AggregateEvent, AggregateState, Handler},
};

#[async_trait]
pub trait StaticHandler {
    type Command;
    type Aggregate: Aggregate;
    type Error;

    fn as_handler() -> AsHandler<Self>
    where
        Self: Sized,
    {
        AsHandler(std::marker::PhantomData)
    }

    async fn handle(
        state: &AggregateState<Self::Aggregate>,
        command: Self::Command,
    ) -> Result<Vec<AggregateEvent<Self::Aggregate>>, Self::Error>;
}

pub struct AsHandler<T>(std::marker::PhantomData<T>);

#[async_trait]
impl<T: StaticHandler> Handler for AsHandler<T>
where
    T: Send + Sync,
    T::Command: Send + Sync,
    AggregateState<T::Aggregate>: Send + Sync,
{
    type Command = T::Command;
    type Aggregate = T::Aggregate;
    type Error = T::Error;

    async fn handle(
        &self,
        state: &AggregateState<Self::Aggregate>,
        command: Self::Command,
    ) -> Result<Vec<AggregateEvent<Self::Aggregate>>, Self::Error> {
        T::handle(state, command).await
    }
}
