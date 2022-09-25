use std::marker::PhantomData;

use async_trait::async_trait;
use eventually::{event, message::Message, version, version::Version};

#[derive(Debug, Clone)]
pub struct EventStore<Id, Evt> {
    id_type: PhantomData<Id>,
    evt_type: PhantomData<Evt>,
}

#[async_trait]
impl<Id, Evt> event::Store for EventStore<Id, Evt>
where
    Id: Send + Sync,
    Evt: Message + Send + Sync,
{
    type StreamId = Id;
    type Event = Evt;
    type StreamError = ();
    type AppendError = Option<version::ConflictError>;

    fn stream(
        &self,
        id: &Self::StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<Self::StreamId, Self::Event, Self::StreamError> {
        todo!()
    }

    async fn append(
        &self,
        id: Self::StreamId,
        version_check: event::StreamVersionExpected,
        events: Vec<event::Envelope<Self::Event>>,
    ) -> Result<Version, Self::AppendError> {
        todo!()
    }
}
