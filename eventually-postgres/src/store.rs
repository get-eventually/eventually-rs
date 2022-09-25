use std::{marker::PhantomData, string::ToString};

use async_trait::async_trait;
use eventually::{event, message::Message, version, version::Version};

use crate::serde::Serde;

#[derive(Debug, Clone)]
pub struct EventStore<Id, Evt, OutEvt, S>
where
    OutEvt: From<Evt>,
    S: Serde<OutEvt>,
{
    id_type: PhantomData<Id>,
    evt_type: PhantomData<Evt>,
    out_evt_type: PhantomData<OutEvt>,
    serde: S,
}

#[async_trait]
impl<Id, Evt, OutEvt, S> event::Store for EventStore<Id, Evt, OutEvt, S>
where
    Id: ToString + Send + Sync,
    Evt: TryFrom<OutEvt> + Message + Send + Sync,
    OutEvt: From<Evt> + Send + Sync,
    S: Serde<OutEvt> + Send + Sync,
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
