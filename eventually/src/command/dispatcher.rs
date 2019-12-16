use std::{future::Future, pin::Pin};

use futures::StreamExt;

use crate::{
    aggregate,
    aggregate::{Aggregate, AggregateExt},
    command,
    command::Handler as CommandHandler,
    store::{ReadStore, WriteStore},
};

pub type SourceIdOf<I: Identifiable> = I::SourceId;

pub trait Identifiable {
    type SourceId: Eq;

    fn source_id(&self) -> Self::SourceId;
}

#[derive(Debug, PartialEq)]
pub enum Error<A, C, RS> {
    RecreateStateFailed(A),
    CommandFailed(C),
    ApplyStateFailed(A),
    AppendEventsFailed(RS),
}

impl<A, C, RS> std::error::Error for Error<A, C, RS>
where
    A: std::error::Error,
    C: std::error::Error,
    RS: std::error::Error,
{
}

impl<A, C, RS> std::fmt::Display for Error<A, C, RS>
where
    A: std::error::Error,
    C: std::error::Error,
    RS: std::error::Error,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::RecreateStateFailed(e) => write!(f, "Failed to recreate state: {}", e),
            Error::CommandFailed(e) => write!(f, "Failed to handle command: {}", e),
            Error::ApplyStateFailed(e) => write!(f, "Failed to apply new events: {}", e),
            Error::AppendEventsFailed(e) => write!(f, "Failed to append events to store: {}", e),
        }
    }
}

pub struct Dispatcher<Store, Handler> {
    store: Store,
    handler: Handler,
}

impl<Store, Handler> Dispatcher<Store, Handler>
where
    Handler: CommandHandler + Send,
    Store: WriteStore + Send,
    <Store as ReadStore>::SourceId: Clone + Eq + Send,
    <Store as ReadStore>::Offset: Default + Send,
    <Store as ReadStore>::Stream: Send,
    <Store as WriteStore>::Error: std::error::Error + Send + 'static,
    command::AggregateOf<Handler>: AggregateExt<Event = <Store as ReadStore>::Event> + Send,
    command::CommandOf<Handler>: Identifiable<SourceId = <Store as ReadStore>::SourceId> + Send,
    aggregate::EventOf<command::AggregateOf<Handler>>: Clone + Send,
    aggregate::StateOf<command::AggregateOf<Handler>>: Default + Send,
    aggregate::ErrorOf<command::AggregateOf<Handler>>: std::error::Error + Send + 'static,
    command::ErrorOf<Handler>: std::error::Error + Send + 'static,
{
    #[inline]
    pub fn new(store: Store, handler: Handler) -> Self {
        Dispatcher { store, handler }
    }

    pub fn dispatch(
        &mut self,
        c: command::CommandOf<Handler>,
    ) -> impl Future<
        Output = Result<
            aggregate::StateOf<command::AggregateOf<Handler>>,
            Error<aggregate::ErrorOf<command::AggregateOf<Handler>>, Handler::Error, Store::Error>,
        >,
    > + '_ {
        async move {
            let id = c.source_id();

            let events = self
                .store
                .stream(id.clone(), <Store as ReadStore>::Offset::default());

            let state = command::AggregateOf::<Handler>::async_fold(
                aggregate::StateOf::<command::AggregateOf<Handler>>::default(),
                events,
            )
            .await
            .map_err(Error::RecreateStateFailed)?;

            let new_events = self
                .handler
                .handle(&state, c)
                .await
                .map_err(Error::CommandFailed)?;

            let new_state =
                command::AggregateOf::<Handler>::fold(state, new_events.clone().into_iter())
                    .map_err(Error::ApplyStateFailed)?;

            self.store
                .append(id, <Store as ReadStore>::Offset::default(), new_events)
                .await
                .map_err(Error::AppendEventsFailed)?;

            Ok(new_state)
        }
    }
}
