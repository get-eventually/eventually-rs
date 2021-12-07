use std::{
    collections::HashMap,
    convert::Infallible,
    hash::Hash,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use futures::stream::{iter, StreamExt};

use crate::{
    event,
    event::{Event, PersistedEvents},
    version::{ConflictError, Version},
};

#[derive(Debug)]
struct InMemoryBackend<Id, Evt> {
    event_streams: HashMap<Id, PersistedEvents<Id, Evt>>,
}

impl<Id, Evt> Default for InMemoryBackend<Id, Evt> {
    fn default() -> Self {
        Self {
            event_streams: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InMemory<Id, Evt> {
    backend: Arc<RwLock<InMemoryBackend<Id, Evt>>>,
}

impl<Id, Evt> Default for InMemory<Id, Evt> {
    fn default() -> Self {
        Self {
            backend: Arc::default(),
        }
    }
}

#[async_trait]
impl<Id, Evt> event::Store for InMemory<Id, Evt>
where
    Id: Clone + Eq + Hash + Send + Sync,
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
            .get(id)
            .cloned()
            .unwrap_or_default() // NOTE: the new Vec is empty, so there will be no memory allocation!
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
        events: Vec<Event<Self::Event>>,
    ) -> Result<Version, Self::AppendError> {
        let mut backend = self
            .backend
            .write()
            .expect("acquire write lock on event store backend");

        let last_event_stream_version = backend
            .event_streams
            .get(&id)
            .and_then(|events| events.last())
            .map(|event| event.version)
            .unwrap_or_default();

        if let event::StreamVersionExpected::MustBe(expected) = version_check {
            if last_event_stream_version != expected {
                return Err(ConflictError {
                    expected,
                    actual: last_event_stream_version,
                });
            }
        }

        let mut persisted_events: PersistedEvents<Id, Evt> = events
            .into_iter()
            .enumerate()
            .map(|(i, payload)| event::Persisted {
                stream_id: id.clone(),
                version: last_event_stream_version + (i as u64) + 1,
                payload,
            })
            .collect();

        let new_last_event_stream_version = persisted_events
            .last()
            .map(|evt| evt.version)
            .unwrap_or_default();

        backend
            .event_streams
            .entry(id)
            .and_modify(|events| events.append(&mut persisted_events))
            .or_insert_with(|| persisted_events);

        Ok(new_last_event_stream_version)
    }
}

#[derive(Debug, Clone)]
pub struct Tracking<S>
where
    S: event::Store,
{
    store: S,

    #[allow(clippy::type_complexity)] // It is a complex type but still readable.
    events: Arc<RwLock<Vec<event::Persisted<S::StreamId, S::Event>>>>,
}

impl<S> Tracking<S>
where
    S: event::Store,
    S::StreamId: Clone,
    S::Event: Clone,
{
    pub fn recorded_events(&self) -> Vec<event::Persisted<S::StreamId, S::Event>> {
        self.events
            .read()
            .expect("acquire lock on recorded events list")
            .clone()
    }

    pub fn reset_recorded_events(&self) {
        self.events
            .write()
            .expect("acquire lock on recorded events list")
            .clear();
    }
}

#[async_trait]
impl<S> event::Store for Tracking<S>
where
    S: event::Store,
    S::StreamId: Clone,
    S::Event: Clone,
{
    type StreamId = S::StreamId;
    type Event = S::Event;
    type StreamError = S::StreamError;
    type AppendError = S::AppendError;

    fn stream(
        &self,
        id: &Self::StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<Self::StreamId, Self::Event, Self::StreamError> {
        self.store.stream(id, select)
    }

    async fn append(
        &self,
        id: Self::StreamId,
        version_check: event::StreamVersionExpected,
        events: Vec<Event<Self::Event>>,
    ) -> Result<Version, Self::AppendError> {
        let new_version = self
            .store
            .append(id.clone(), version_check, events.clone())
            .await?;

        let events_size = events.len();
        let previous_version = new_version - (events_size as Version);

        let mut persisted_events = events
            .into_iter()
            .enumerate()
            .map(|(i, evt)| event::Persisted {
                stream_id: id.clone(),
                version: previous_version + (i as Version) + 1,
                payload: evt,
            })
            .collect();

        self.events
            .write()
            .expect("acquire lock on recorded events list")
            .append(&mut persisted_events);

        Ok(new_version)
    }
}

pub trait EventStoreExt: event::Store + Sized {
    fn with_recorded_events_tracking(self) -> Tracking<Self> {
        Tracking {
            store: self,
            events: Arc::default(),
        }
    }
}

impl<T> EventStoreExt for T where T: event::Store {}

#[allow(clippy::semicolon_if_nothing_returned)] // False positives :shrugs:
#[cfg(test)]
mod test {
    use futures::TryStreamExt;

    use super::*;
    use crate::{event, event::Store, version::Version};

    #[tokio::test]
    async fn it_works() {
        let event_store = InMemory::<&'static str, &'static str>::default();

        let stream_id = "stream:test";
        let events = vec![
            event::Event::from("event-1"),
            event::Event::from("event-2"),
            event::Event::from("event-3"),
        ];

        let new_event_stream_version = event_store
            .append(
                stream_id,
                event::StreamVersionExpected::MustBe(0),
                events.clone(),
            )
            .await
            .expect("append should not fail");

        let expected_version = events.len() as Version;
        assert_eq!(expected_version, new_event_stream_version);

        let expected_events = events
            .into_iter()
            .enumerate()
            .map(|(i, payload)| event::Persisted {
                stream_id,
                version: (i as Version) + 1,
                payload,
            })
            .collect::<Vec<_>>();

        let event_stream: Vec<_> = event_store
            .stream(&stream_id, event::VersionSelect::All)
            .try_collect()
            .await
            .expect("opening an event stream should not fail");

        assert_eq!(expected_events, event_stream);
    }

    #[tokio::test]
    async fn tracking_store_works() {
        let event_store = InMemory::<&'static str, &'static str>::default();
        let tracking_event_store = event_store.with_recorded_events_tracking();

        let stream_id = "stream:test";
        let events = vec![
            event::Event::from("event-1"),
            event::Event::from("event-2"),
            event::Event::from("event-3"),
        ];

        tracking_event_store
            .append(
                stream_id,
                event::StreamVersionExpected::MustBe(0),
                events.clone(),
            )
            .await
            .expect("append should not fail");

        let event_stream: Vec<_> = tracking_event_store
            .stream(&stream_id, event::VersionSelect::All)
            .try_collect()
            .await
            .expect("opening an event stream should not fail");

        assert_eq!(event_stream, tracking_event_store.recorded_events());
    }

    #[tokio::test]
    async fn version_conflict_checks_work_as_expected() {
        let event_store = InMemory::<&'static str, &'static str>::default();

        let stream_id = "stream:test";
        let events = vec![
            event::Event::from("event-1"),
            event::Event::from("event-2"),
            event::Event::from("event-3"),
        ];

        let append_error = event_store
            .append(
                stream_id,
                event::StreamVersionExpected::MustBe(3),
                events.clone(),
            )
            .await
            .expect_err("the event stream version should be zero");

        assert_eq!(
            ConflictError {
                expected: 3,
                actual: 0,
            },
            append_error
        );
    }
}
