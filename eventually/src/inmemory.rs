use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use futures::future::BoxFuture;
use futures::stream::{iter, StreamExt};

use crate::eventstore::{
    ConflictError, EventStore, EventStream, PersistedEvent, Select, StreamInstance, StreamName,
    VersionCheck,
};

#[derive(Debug, Clone)]
pub struct InMemoryEventStore<T>
where
    T: Debug,
{
    inner: Arc<RwLock<EventStoreDataSource<T>>>,
}

impl<T> Default for InMemoryEventStore<T>
where
    T: Debug,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(EventStoreDataSource {
                global_event_stream: Vec::new(),
                indexed_event_streams: HashMap::new(),
            })),
        }
    }
}

#[derive(Debug, Default)]
struct EventStoreDataSource<T>
where
    T: Debug,
{
    global_event_stream: Vec<PersistedEvent<T>>,
    indexed_event_streams: HashMap<String, HashMap<String, Vec<usize>>>,
}

impl<T> EventStore for InMemoryEventStore<T>
where
    T: Debug + Clone + Send + Sync,
{
    type Event = T;
    type AppendError = ConflictError;
    type StreamError = Infallible;

    fn append<'a>(
        &'a mut self,
        stream: StreamInstance<'a>,
        expected: VersionCheck,
        events: Vec<Self::Event>,
    ) -> BoxFuture<'a, Result<u64, Self::AppendError>> {
        let f = async move {
            let mut inner = self.inner.write().expect("acquire write lock");

            let current_version = inner
                .indexed_event_streams
                .get(stream.typ())
                .and_then(|event_streams| event_streams.get(stream.name()))
                .map(Vec::len)
                .unwrap_or_default();

            expected.check(current_version as u64)?;

            let mut events: Vec<PersistedEvent<Self::Event>> = events
                .into_iter()
                .enumerate()
                .map(|(i, event)| PersistedEvent {
                    stream_type: stream.typ().to_owned(),
                    stream_name: stream.name().to_owned(),
                    version: (current_version + i + 1) as u64,
                    event,
                })
                .collect();

            let next_global_offset = inner.global_event_stream.len();
            let new_events_len = events.len();
            let new_version = current_version + new_events_len;

            inner.global_event_stream.append(&mut events);

            let mut new_events_offsets: Vec<usize> =
                (next_global_offset..next_global_offset + new_events_len).collect();

            inner
                .indexed_event_streams
                .entry(stream.typ().to_owned())
                .or_insert_with(HashMap::new)
                .entry(stream.name().to_owned())
                .and_modify(|stream| {
                    stream.append(&mut new_events_offsets);
                })
                .or_insert_with(|| new_events_offsets);

            Ok(new_version as u64)
        };

        Box::pin(f)
    }

    fn stream(
        &self,
        stream: StreamName,
        select: Select,
    ) -> EventStream<Self::Event, Self::StreamError> {
        let inner = self.inner.read().expect("aquire read lock");
        let global_event_stream = inner.global_event_stream.clone();
        let indexed_event_streams = inner.indexed_event_streams.clone();

        match stream {
            StreamName::All => iter(
                global_event_stream
                    .into_iter()
                    .enumerate()
                    .filter_map(move |(i, event)| match select {
                        Select::From(v) if (i as u64) < v => None,
                        _ => Some(event),
                    })
                    .map(Ok),
            )
            .boxed(),

            StreamName::Type(typ) => iter(
                indexed_event_streams
                    .get(typ)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .flat_map(|(_, streams)| streams.into_iter())
                    .map(move |i| (i, global_event_stream[i].clone()))
                    .filter_map(move |(i, event)| match select {
                        Select::From(v) if (i as u64) < v => None,
                        _ => Some(event),
                    })
                    .map(Ok),
            )
            .boxed(),

            StreamName::Instance(stream) => iter(
                indexed_event_streams
                    .get(stream.typ())
                    .unwrap()
                    .get(stream.name())
                    .unwrap()
                    .clone()
                    .into_iter()
                    .map(move |i| global_event_stream[i].clone())
                    .filter(move |event| match select {
                        Select::All => true,
                        Select::From(v) => event.version >= v,
                    })
                    .map(Ok),
            )
            .boxed(),
        }
    }

    fn subscribe(stream: StreamName) -> EventStream<Self::Event, Self::StreamError> {
        todo!()
    }
}
