use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::Display,
    sync::{atomic::AtomicU64, Arc, RwLock},
};

use async_trait::async_trait;
use futures::stream::{iter, StreamExt};

use crate::{
    event,
    event::{Events, PersistedEvents},
    version::{ConflictError, Version},
};

#[derive(Debug)]
struct InMemoryEventStoreBackend<Id, Evt> {
    event_streams: HashMap<String, PersistedEvents<Id, Evt>>,
}

impl<Id, Evt> Default for InMemoryEventStoreBackend<Id, Evt> {
    fn default() -> Self {
        Self {
            event_streams: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryEventStore<Id, Evt> {
    global_offset: Arc<AtomicU64>,
    backend: Arc<RwLock<InMemoryEventStoreBackend<Id, Evt>>>,
}

#[async_trait]
impl<Id, Evt> event::Store for InMemoryEventStore<Id, Evt>
where
    Id: Clone + Display + Send + Sync,
    Evt: Clone + Send + Sync,
{
    type StreamId = Id;
    type Event = Evt;
    type StreamError = Infallible;
    type AppendError = ConflictError;

    fn stream(
        &self,
        id: &Self::StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<Self::StreamId, Self::Event, Self::StreamError> {
        let backend = self
            .backend
            .read()
            .expect("acquire read lock on event store backend");

        let events = backend
            .event_streams
            .get(&id.to_string())
            .cloned()
            .unwrap_or(Vec::new()) // NOTE: the new Vec is empty, so there will be no memory allocation!
            .into_iter()
            .filter(move |evt| match select {
                event::VersionSelect::All => true,
                event::VersionSelect::From(v) => evt.version >= v,
            });

        iter(events).map(Ok).boxed()
    }

    async fn append(
        &self,
        id: Self::StreamId,
        version_check: event::StreamVersionExpected,
        events: Events<Self::Event>,
    ) -> Result<Version, Self::AppendError> {
        let mut backend = self
            .backend
            .write()
            .expect("acquire write lock on event store backend");

        let event_stream_id_string = id.to_string();

        let last_event_stream_version = backend
            .event_streams
            .get(&event_stream_id_string)
            .and_then(|events| events.last())
            .map(|event| event.version)
            .unwrap_or_default();

        if let event::StreamVersionExpected::Exact(expected_event_stream_version) = version_check {
            if last_event_stream_version != expected_event_stream_version {
                return Err(ConflictError {
                    expected: expected_event_stream_version,
                    actual: last_event_stream_version,
                });
            }
        }

        let mut persisted_events: PersistedEvents<Id, Evt> = events
            .into_iter()
            .enumerate()
            // TODO: add sequence number
            .map(|(i, evt)| event::Persisted {
                stream_id: id.clone(),
                version: last_event_stream_version + (i as u64) + 1,
                inner: evt,
            })
            .collect();

        let new_last_event_stream_version = persisted_events
            .last()
            .map(|evt| evt.version)
            .unwrap_or_default();

        backend
            .event_streams
            .entry(event_stream_id_string)
            .and_modify(|events| events.append(&mut persisted_events))
            .or_insert_with(|| persisted_events);

        Ok(new_last_event_stream_version)
    }
}
