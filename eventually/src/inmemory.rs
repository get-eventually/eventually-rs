use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use async_stream::try_stream;

use async_trait::async_trait;

use futures::stream::{iter, StreamExt};

use serde::ser::{Serialize, SerializeSeq, Serializer};

use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::eventstore::{
    with_global_sequence_number, ConflictError, EventStore, EventStream, PersistedEvent, Select,
    Stream, VersionCheck,
};

use crate::Events;

#[derive(Debug, Clone)]
pub struct InMemoryEventStore<T>
where
    T: Debug,
{
    inner: Arc<RwLock<EventStoreDataSource<T>>>,
}

impl<T> InMemoryEventStore<T>
where
    T: Debug + Clone,
{
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = channel(buffer_size);

        Self {
            inner: Arc::new(RwLock::new(EventStoreDataSource {
                tx,
                rx,
                global_event_stream: Vec::new(),
                indexed_event_streams: HashMap::new(),
            })),
        }
    }
}

impl<T> Default for InMemoryEventStore<T>
where
    T: Debug + Clone,
{
    #[inline]
    fn default() -> Self {
        Self::new(128)
    }
}

impl<T> Serialize for InMemoryEventStore<T>
where
    T: Debug + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data_source = self.inner.read().unwrap();
        let mut event_stream =
            serializer.serialize_seq(Some(data_source.global_event_stream.len()))?;

        for event in &data_source.global_event_stream {
            event_stream.serialize_element(event)?;
        }
        event_stream.end()
    }
}

#[derive(Debug)]
struct EventStoreDataSource<T>
where
    T: Debug,
{
    global_event_stream: Vec<PersistedEvent<T>>,
    indexed_event_streams: HashMap<String, HashMap<String, Vec<usize>>>,
    tx: Sender<PersistedEvent<T>>,
    // NOTE: keeping a reference to rx so that tx won't return an error
    // when inserting new events in the channel.
    rx: Receiver<PersistedEvent<T>>,
}

#[async_trait]
impl<T> EventStore for InMemoryEventStore<T>
where
    T: Debug + Clone + Send + Sync + Unpin,
{
    type Event = T;
    type AppendError = ConflictError;
    type StreamError = Infallible;

    async fn append(
        &mut self,
        category: &str,
        stream_id: &str,
        expected: VersionCheck,
        events: Events<Self::Event>,
    ) -> Result<i64, Self::AppendError> {
        let mut inner = self.inner.write().expect("acquire write lock");

        let current_sequence_number = inner.global_event_stream.len();

        let current_version = inner
            .indexed_event_streams
            .get(category)
            .and_then(|event_streams| event_streams.get(stream_id))
            .map(Vec::len)
            .unwrap_or_default();

        expected.check(current_version as i64)?;

        let mut events: Vec<PersistedEvent<Self::Event>> = events
            .into_iter()
            .enumerate()
            .map(|(i, event)| PersistedEvent {
                stream_category: category.to_owned(),
                stream_id: stream_id.to_owned(),
                version: (current_version + i + 1) as i64,
                event: with_global_sequence_number(event, (current_sequence_number + i) as i64),
            })
            .collect();

        let events_cloned = events.clone();
        let new_events_len = events.len();
        let new_version = current_version + new_events_len;

        inner.global_event_stream.append(&mut events);

        let mut new_events_offsets: Vec<usize> =
            (current_sequence_number..current_sequence_number + new_events_len).collect();

        inner
            .indexed_event_streams
            .entry(category.to_owned())
            .or_insert_with(HashMap::new)
            .entry(stream_id.to_owned())
            .and_modify(|stream| {
                stream.append(&mut new_events_offsets);
            })
            .or_insert_with(|| new_events_offsets);

        for event in events_cloned.into_iter() {
            inner.tx.send(event).unwrap();
        }

        Ok(new_version as i64)
    }

    fn stream(
        &self,
        stream: Stream,
        select: Select,
    ) -> EventStream<Self::Event, Self::StreamError> {
        let inner = self.inner.read().expect("aquire read lock");
        let global_event_stream = inner.global_event_stream.clone();
        let indexed_event_streams = inner.indexed_event_streams.clone();

        match stream {
            Stream::All => iter(
                global_event_stream
                    .into_iter()
                    .enumerate()
                    .filter_map(move |(i, event)| match select {
                        Select::From(v) if (i as i64) < v => None,
                        _ => Some(event),
                    })
                    .map(Ok),
            )
            .boxed(),

            Stream::Category(id) => iter(
                indexed_event_streams
                    .get(id)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .flat_map(|(_, streams)| streams.into_iter())
                    .map(move |i| (i, global_event_stream[i].clone()))
                    .filter_map(move |(i, event)| match select {
                        Select::From(v) if (i as i64) < v => None,
                        _ => Some(event),
                    })
                    .map(Ok),
            )
            .boxed(),

            Stream::Id { category, id } => iter(
                indexed_event_streams
                    .get(category)
                    .unwrap()
                    .get(id)
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

    fn subscribe(&self, stream: Stream) -> EventStream<Self::Event, Self::StreamError> {
        let inner = self.inner.read().expect("acquire read lock");
        let mut rx = inner.tx.subscribe();
        let owned_stream = OwnedStream::from(stream);

        Box::pin(try_stream! {
            while let Ok(event) = rx.recv().await {
                let PersistedEvent{ stream_id, stream_category, .. } = &event;

                match &owned_stream {
                    OwnedStream::All => yield event,
                    OwnedStream::Category(category) if stream_category == category => yield event,
                    OwnedStream::Id { category, id } if stream_category == category && stream_id == id => yield event,
                    _ => continue,
                }
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OwnedStream {
    All,
    Category(String),
    Id { id: String, category: String },
}

impl<'a> From<Stream<'a>> for OwnedStream {
    #[inline]
    fn from(stream: Stream<'a>) -> Self {
        match stream {
            Stream::All => Self::All,
            Stream::Category(category) => Self::Category(category.to_owned()),
            Stream::Id { id, category } => Self::Id {
                id: id.to_owned(),
                category: category.to_owned(),
            },
        }
    }
}
