use std::{future::Future, pin::Pin};

use futures::{Stream, StreamExt};

use crate::aggregate::{
    optional::{AsAggregate as OptionalAsAggregate, OptionalAggregate},
    referential::{AsAggregate as ReferentialAsAggregate, ReferentialAggregate},
    versioned::AsAggregate as VersionedAggregate,
    Aggregate,
};

pub trait AggregateExt: Aggregate {
    /// Applies a stream of events to the current `Aggregate` state, returning
    /// the updated state or an error, if any such happened.
    fn fold(
        mut state: Self::State,
        events: impl Iterator<Item = Self::Event>,
    ) -> Result<Self::State, Self::Error> {
        events.fold(Ok(state), |previous, event| {
            previous.and_then(|state| Self::apply(state, event))
        })
    }

    fn async_fold<'a>(
        mut state: Self::State,
        events: impl Stream<Item = Self::Event> + Send + 'a,
    ) -> Pin<Box<dyn Future<Output = Result<Self::State, Self::Error>> + Send + 'a>>
    where
        Self::State: Send + 'a,
        Self::Event: Send + 'a,
        Self::Error: Send + 'a,
    {
        Box::pin(events.fold(Ok(state), |previous, event| {
            async move { previous.and_then(|state| Self::apply(state, event)) }
        }))
    }
}

impl<T: Aggregate> AggregateExt for T {}
