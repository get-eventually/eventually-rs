pub mod dispatcher;
pub mod r#static;

use std::future::Future;

use crate::aggregate::{Aggregate, EventOf, StateOf};

pub type CommandOf<H: Handler> = H::Command;
pub type AggregateOf<H: Handler> = H::Aggregate;
pub type ErrorOf<H: Handler> = H::Error;

pub trait Handler {
    type Command;
    type Aggregate: Aggregate;
    type Error;
    type Result: Future<Output = Result<Vec<EventOf<Self::Aggregate>>, Self::Error>>;

    fn handle(&self, state: &StateOf<Self::Aggregate>, command: Self::Command) -> Self::Result;
}
