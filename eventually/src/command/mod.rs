pub mod dispatcher;
pub mod r#static;

use std::result::Result as StdResult;

use async_trait::async_trait;

use crate::aggregate::{Aggregate, EventOf, StateOf};

pub type CommandOf<H: Handler> = H::Command;
pub type AggregateOf<H: Handler> = H::Aggregate;
pub type ErrorOf<H: Handler> = H::Error;

pub type Result<Event, Error> = StdResult<Vec<Event>, Error>;

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
