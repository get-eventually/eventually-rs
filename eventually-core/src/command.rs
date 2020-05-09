use futures::future::BoxFuture;

use crate::aggregate::{Aggregate, EventOf, StateOf};

pub type CommandOf<H> = <H as Handler>::Command;
pub type AggregateOf<H> = <H as Handler>::Aggregate;
pub type ErrorOf<H> = <H as Handler>::Error;

pub type Result<Event, Error> = std::result::Result<Vec<Event>, Error>;

pub trait Handler {
    type Command;
    type Aggregate: Aggregate;
    type Error;

    fn handle<'a, 'b: 'a>(
        &'a self,
        state: &'b StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<EventOf<Self::Aggregate>, Self::Error>>;
}
