use async_trait::async_trait;

use crate::aggregate::{Aggregate, EventOf, StateOf};

pub type CommandOf<H> = <H as Handler>::Command;
pub type AggregateOf<H> = <H as Handler>::Aggregate;
pub type ErrorOf<H> = <H as Handler>::Error;

pub type Result<Event, Error> = std::result::Result<Vec<Event>, Error>;

#[async_trait]
pub trait Handler {
    type Command;
    type Aggregate: Aggregate;
    type Error;

    async fn handle(
        &self,
        state: &StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> Result<EventOf<Self::Aggregate>, Self::Error>;
}
