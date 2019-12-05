pub mod r#static;

use async_trait::async_trait;

use crate::aggregate::{Aggregate, EventOf, StateOf};

#[async_trait]
pub trait Handler {
    type Command;
    type Aggregate: Aggregate;
    type Error;

    async fn handle(
        &self,
        state: &StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> Result<Vec<EventOf<Self::Aggregate>>, Self::Error>;
}
