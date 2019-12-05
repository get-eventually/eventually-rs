pub mod r#static;

use std::future::Future;

use crate::aggregate::{Aggregate, EventOf, StateOf};

pub trait Handler {
    type Command;
    type Aggregate: Aggregate;
    type Error;
    type Result: Future<Output = Result<Vec<EventOf<Self::Aggregate>>, Self::Error>>;

    fn handle(&self, state: &StateOf<Self::Aggregate>, command: Self::Command) -> Self::Result;
}
