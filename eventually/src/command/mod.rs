pub mod r#static;

use async_trait::async_trait;

use crate::aggregate::Aggregate;

pub type AggregateState<A: Aggregate> = A::State;
pub type AggregateEvent<A: Aggregate> = A::Event;

#[async_trait]
pub trait Handler {
    type Command;
    type Aggregate: Aggregate;
    type Error;

    async fn handle(
        &self,
        state: &AggregateState<Self::Aggregate>,
        command: Self::Command,
    ) -> Result<Vec<AggregateEvent<Self::Aggregate>>, Self::Error>;
}
