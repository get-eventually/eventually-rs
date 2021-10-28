use std::convert::TryFrom;
use std::fmt::{Debug, Display};

use eventually::store::{
    AppendError, EventStream as StoreEventStream, Expected, Persisted, Select,
};

use futures::future::BoxFuture;
use futures::stream::{StreamExt, TryStreamExt};

use lazy_static::lazy_static;

use redis::RedisError;

use serde::{Deserialize, Serialize};

use crate::stream;

static APPEND_TO_STORE_SOURCE: &str = std::include_str!("append_to_store.lua");

lazy_static! {
    static ref APPEND_TO_STORE_SCRIPT: redis::Script = redis::Script::new(APPEND_TO_STORE_SOURCE);
}

/// Result returning the crate [`StoreError`] type.
///
/// [`StoreError`]: enum.Error.html
pub type StoreResult<T> = Result<T, StoreError>;

/// Error types returned by the [`eventually::EventStore`] implementation
/// on the [`EventStore`] type.
///
/// [`eventually::EventStore`]: ../eventually/trait.EventStore.html
/// [`EventStore`]: struct.EventStore.html
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Error returned when failed to encode events to JSON during [`append`].
    ///
    /// [`append`]: struct.EventStore.html#tymethod.append
    #[error("failed to encode events: {0}")]
    EncodeEvents(#[source] serde_json::Error),

    /// Error returned when failed to decoding events from JSON
    /// during either [`stream`] or [`stream_all`].
    ///
    /// [`stream`]: struct.EventStore.html#tymethod.stream
    /// [`stream_all`]: struct.EventStore.html#tymethod.stream_all
    #[error("failed to decode events: {0}")]
    DecodeEvents(#[source] stream::ToPersistedError),

    /// Error returned when failed to decoding events from JSON
    /// during either [`stream`] or [`stream_all`].
    ///
    /// [`stream`]: struct.EventStore.html#tymethod.stream
    /// [`stream_all`]: struct.EventStore.html#tymethod.stream_all
    #[error("failed to decode from JSON: {0}")]
    DecodeJSON(#[source] serde_json::Error),

    /// Error returned when reading the stream coming from `XRANGE .. COUNT n`
    /// during either [`stream`] or [`stream_all`].
    ///
    /// [`stream`]: struct.EventStore.html#tymethod.stream
    /// [`stream_all`]: struct.EventStore.html#tymethod.stream_all
    #[error("failed while reading stream from Redis: {0}")]
    Stream(#[source] RedisError),

    /// Error returned when attempting to read a key from the Redis stream
    /// that does not exist.
    #[error("no key from Redis result: `{0}`")]
    NoKey(&'static str),
}

impl AppendError for StoreError {
    #[inline]
    fn is_conflict_error(&self) -> bool {
        false
    }
}

/// Redis backend implementation for [`eventually::EventStore`] trait.
///
/// [`eventually::EventStore`]: ../eventually/trait.EventStore.html
#[derive(Clone)]
pub struct EventStore<Id, Event> {
    pub(crate) stream_name: &'static str,
    pub(crate) conn: redis::aio::MultiplexedConnection,
    pub(crate) stream_page_size: usize,
    pub(crate) id: std::marker::PhantomData<Id>,
    pub(crate) event: std::marker::PhantomData<Event>,
}

impl<Id, Event> eventually::EventStore for EventStore<Id, Event>
where
    Id: TryFrom<String> + Display + Eq + Clone + Send + Sync,
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    for<'de> Event: Serialize + Send + Sync + Deserialize<'de>,
{
    type SourceId = Id;
    type Event = Event;
    type Error = StoreError;

    fn append(
        &mut self,
        id: Self::SourceId,
        version: Expected,
        events: Vec<Self::Event>,
    ) -> BoxFuture<StoreResult<u32>> {
        let fut = async move {
            let events = events
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()
                .map_err(StoreError::EncodeEvents)?;

            Ok(APPEND_TO_STORE_SCRIPT
                .key(self.stream_name)
                .key(id.to_string())
                .arg(match version {
                    Expected::Any => -1,
                    Expected::Exact(v) => i64::from(v),
                })
                .arg(events)
                .invoke_async(&mut self.conn)
                .await
                .unwrap())
        };

        Box::pin(fut)
    }

    fn stream(
        &self,
        id: Self::SourceId,
        select: Select,
    ) -> BoxFuture<StoreResult<StoreEventStream<Self>>> {
        let fut = async move {
            let stream_name = format!("{}.{}", self.stream_name, id);

            let paginator = stream::into_xrange_stream(
                self.conn.clone(),
                stream_name,
                self.stream_page_size,
                match select {
                    Select::All => 0,
                    Select::From(v) => v as usize,
                },
            );

            Ok(paginator
                .map_err(StoreError::Stream)
                .map(move |res| res.map(|v| (id.clone(), v)))
                .and_then(move |(id, entry)| async move {
                    let event: Vec<u8> = entry.get("event").ok_or(StoreError::NoKey("event"))?;
                    let event: Event =
                        serde_json::from_slice(&event).map_err(StoreError::DecodeJSON)?;

                    let sequence_number: u32 = entry
                        .get("sequence_number")
                        .ok_or(StoreError::NoKey("sequence_number"))?;

                    let version = stream::parse_version(&entry.id);

                    Ok(Persisted::from(id, event)
                        .sequence_number(sequence_number)
                        .version(version as u32))
                })
                .boxed())
        };

        Box::pin(fut)
    }

    fn stream_all(&self, select: Select) -> BoxFuture<StoreResult<StoreEventStream<Self>>> {
        let fut = async move {
            let paginator = stream::into_xrange_stream(
                self.conn.clone(),
                self.stream_name.to_owned(),
                self.stream_page_size,
                match select {
                    Select::All => 0,
                    Select::From(v) => v as usize,
                },
            );

            Ok(paginator
                .map_err(StoreError::Stream)
                .and_then(|entry| async move {
                    Persisted::<Id, Event>::try_from(stream::ToPersisted::from(entry))
                        .map_err(StoreError::DecodeEvents)
                })
                .boxed())
        };

        Box::pin(fut)
    }

    fn remove(&mut self, _id: Self::SourceId) -> BoxFuture<StoreResult<()>> {
        unimplemented!()
    }
}
