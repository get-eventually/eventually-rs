use std::convert::Infallible;

use async_stream::try_stream;

use async_trait::async_trait;

use futures::stream::{Stream, StreamExt};

use crate::eventstore::{EventStore, EventStream, PersistedEvent, Select, Stream as StreamId};

#[async_trait]
pub trait Subscription: Send + Sync {
    type Event;
    type Error;
    type CheckpointError;

    fn start(&self) -> EventStream<Self::Event, Self::Error>;
    async fn checkpoint(&mut self, sequence_number: i64) -> Result<(), Self::CheckpointError>;
}

#[derive(Debug, Clone)]
pub struct VolatileSubscription<ES> {
    event_stream_id: StreamId<'static>,
    event_store: ES,
}

impl<ES> VolatileSubscription<ES> {
    #[inline]
    pub fn new(event_store: ES, event_stream_id: StreamId<'static>) -> Self {
        Self {
            event_store,
            event_stream_id,
        }
    }
}

#[async_trait]
impl<ES> Subscription for VolatileSubscription<ES>
where
    ES: EventStore,
{
    type Event = ES::Event;
    type Error = ES::StreamError;
    type CheckpointError = Infallible;

    #[inline]
    fn start(&self) -> EventStream<Self::Event, Self::Error> {
        self.event_store.subscribe(self.event_stream_id)
    }

    #[inline]
    async fn checkpoint(&mut self, sequence_number: i64) -> Result<(), Self::CheckpointError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CatchUpSubscription<ES> {
    event_stream_id: StreamId<'static>,
    event_store: ES,
}

impl<ES> CatchUpSubscription<ES>
where
    ES: EventStore,
{
    #[inline]
    pub fn new(event_store: ES, event_stream_id: StreamId<'static>) -> Self {
        Self {
            event_store,
            event_stream_id,
        }
    }

    fn start_stream(
        &self,
    ) -> impl Stream<Item = Result<PersistedEvent<ES::Event>, ES::StreamError>> + '_
    where
        ES::Event: Unpin,
        ES::StreamError: Unpin,
    {
        try_stream! {
            // TODO: ask a checkpointer for the latest checkpoint
            let checkpoint = 0i64;

            let subscription = self.event_store.subscribe(self.event_stream_id);
            let one_off_stream = self
                .event_store
                .stream(self.event_stream_id, Select::From(checkpoint));

            let mut event_stream = one_off_stream.chain(subscription);

            while let Some(result) = event_stream.next().await {
                // TODO: filter events that have already been processed,
                // using the global sequence number.
                let event = result?;
                yield event;
            }
        }
    }
}

#[async_trait]
impl<ES> Subscription for CatchUpSubscription<ES>
where
    ES: EventStore,
    ES::Event: Unpin,
    ES::StreamError: Unpin,
{
    type Event = ES::Event;
    type Error = ES::StreamError;
    type CheckpointError = Infallible;

    #[inline]
    fn start(&self) -> EventStream<Self::Event, Self::Error> {
        Box::pin(self.start_stream())
    }

    async fn checkpoint(&mut self, sequence_number: i64) -> Result<(), Self::CheckpointError> {
        todo!()
    }
}
