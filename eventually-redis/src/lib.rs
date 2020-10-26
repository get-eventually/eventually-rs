//! Redis backend implementation for [`eventually` crate](https://crates.io/crates/eventually).
//!
//! ## Event Store
//!
//! `eventually-redis` supports the [`eventually::EventStore`] trait through
//! the [`EventStore`] type.
//!
//! ## Event Subscriber
//!
//! `eventually-redis` supports the [`eventually::EventSubscriber`] trait
//! through the [`EventSubscriber`] type.
//!
//! [`eventually::EventStore`]: ../eventually/trait.EventStore.html
//! [`EventStore`]: struct.EventStore.html
//! [`EventSubscriber`]: struct.EventSubscriber.html

use std::convert::TryFrom;
use std::fmt::{Debug, Display};

use eventually_core::store::{
    AppendError, EventStream as StoreEventStream, Expected, Persisted, Select,
};
use eventually_core::subscription::EventStream as SubscriberEventStream;

use futures::future::BoxFuture;
use futures::stream::{Stream, StreamExt, TryStreamExt};

use lazy_static::lazy_static;

use redis::streams::{StreamId, StreamRangeReply};
use redis::{AsyncCommands, RedisError, RedisResult};

use serde::{Deserialize, Serialize};

/// Default size of a paginated request to Redis `XRANGE .. COUNT n`
/// for the [`EventStore::stream`] and [`EventStore::stream_all`] operations.
///
/// Page size can be overridden through the [`EventStoreBuilder::stream_page_size`]
/// option.
///
/// [`EventStore::stream`]: struct.EventStore.html#tymethod.stream
/// [`EventStore::stream_all`]: struct.EventStore.html#tymethod.stream_all
/// [`EventStoreBuilder::stream_page_size`]: struct.EventStoreBuilder.html#method.stream_page_size
pub const STREAM_PAGE_DEFAULT: usize = 128;

static APPEND_TO_STORE_SOURCE: &'static str = std::include_str!("append_to_store.lua");

lazy_static! {
    static ref APPEND_TO_STORE_SCRIPT: redis::Script = redis::Script::new(APPEND_TO_STORE_SOURCE);
}

/// Builder type for [`EventStore`] and [`EventSubscriber`] types.
///
/// The same builder instance can be used to build multiple instances of [`EventStore`]
/// and [`EventSubscriber`].
///
/// [`EventStore`]: struct.EventStore.html
/// [`EventSubscriber`]: struct.EventSubscriber.html
#[derive(Clone)]
pub struct EventStoreBuilder {
    client: redis::Client,
    stream_page_size: Option<usize>,
}

impl EventStoreBuilder {
    /// Creates a new builder instance using the specified Redis client.
    pub fn new(client: redis::Client) -> Self {
        Self {
            client,
            stream_page_size: None,
        }
    }

    /// Changes the page size used by the [`Stream`] returned in [`EventStore::stream`]
    /// and [`EventStore::stream_all`].
    ///
    /// [`Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
    /// [`EventStore::stream`]: struct.EventStore.html#tymethod.stream
    /// [`EventStore::stream_all`]: struct.EventStore.html#tymethod.stream_all
    pub fn stream_page_size(mut self, size: usize) -> Self {
        self.stream_page_size = Some(size);
        self
    }

    /// Builds a new [`EventStore`] instance.
    ///
    /// This method returns an `std::future::Future` completing after a
    /// connection with Redis is successfully established.
    ///
    /// [`EventStore`]: struct.EventStore.html
    pub async fn build_store<Id, Event>(
        &self,
        stream_name: &'static str,
    ) -> RedisResult<EventStore<Id, Event>> {
        Ok(EventStore {
            stream_name,
            conn: self.client.get_multiplexed_async_connection().await?,
            stream_page_size: self.stream_page_size.unwrap_or(STREAM_PAGE_DEFAULT),
            id: std::marker::PhantomData,
            event: std::marker::PhantomData,
        })
    }

    /// Builds a new [`EventSubscriber`] instance.
    ///
    /// [`EventSubscriber`]: struct.EventSubscriber.html
    pub fn build_subscriber<Id, Event>(
        &self,
        stream_name: &'static str,
    ) -> EventSubscriber<Id, Event> {
        EventSubscriber {
            stream_name,
            client: self.client.clone(),
            id: std::marker::PhantomData,
            event: std::marker::PhantomData,
        }
    }
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
    DecodeEvents(#[source] serde_json::Error),

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

    /// Error returned when attempting to decode the source id of one
    /// Redis stream entry.
    #[error("failed to decode source_id from Redis entry: {0}")]
    DecodeSourceId(#[source] anyhow::Error),
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
    stream_name: &'static str,
    conn: redis::aio::MultiplexedConnection,
    stream_page_size: usize,
    id: std::marker::PhantomData<Id>,
    event: std::marker::PhantomData<Event>,
}

impl<Id, Event> eventually_core::store::EventStore for EventStore<Id, Event>
where
    Id: TryFrom<String> + Display + Eq + Clone + Send + Sync,
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    Event: Serialize + Send + Sync,
    for<'de> Event: Deserialize<'de>,
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
                    Expected::Exact(v) => v as i64,
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
            let stream_name = format!("{}.{}", self.stream_name, id.to_string());

            let paginator = RedisPaginatedStream {
                conn: self.conn.clone(),
                stream_name,
                page_size: self.stream_page_size,
                from: match select {
                    Select::All => 0,
                    Select::From(v) => v as usize,
                },
            };

            Ok(paginator
                .into_stream()
                .map_err(StoreError::Stream)
                .map(move |res| res.map(|v| (id.clone(), v)))
                .and_then(move |(id, entry)| async move {
                    let event: Vec<u8> = entry
                        .get("event")
                        .ok_or_else(|| StoreError::NoKey("event"))?;
                    let event: Event =
                        serde_json::from_slice(&event).map_err(StoreError::DecodeEvents)?;

                    let (version, sequence_number) = parse_entry_id(&entry.id);

                    Ok(Persisted::from(id, event)
                        .sequence_number(sequence_number as u32)
                        .version(version as u32))
                })
                .boxed())
        };

        Box::pin(fut)
    }

    fn stream_all(&self, select: Select) -> BoxFuture<StoreResult<StoreEventStream<Self>>> {
        let fut = async move {
            let paginator = RedisPaginatedStream {
                conn: self.conn.clone(),
                stream_name: self.stream_name.to_owned(),
                page_size: self.stream_page_size,
                from: match select {
                    Select::All => 0,
                    Select::From(v) => v as usize,
                },
            };

            Ok(paginator
                .into_stream()
                .map_err(StoreError::Stream)
                .and_then(|entry| async move {
                    let source_id: String = entry
                        .get("source_id")
                        .ok_or_else(|| StoreError::NoKey("source_id"))?;

                    let source_id: Id = Id::try_from(source_id)
                        .map_err(anyhow::Error::from)
                        .map_err(StoreError::DecodeSourceId)?;

                    let event: Vec<u8> = entry
                        .get("event")
                        .ok_or_else(|| StoreError::NoKey("event"))?;
                    let event: Event =
                        serde_json::from_slice(&event).map_err(StoreError::DecodeEvents)?;

                    let (sequence_number, version) = parse_entry_id(&entry.id);

                    Ok(Persisted::from(source_id, event)
                        .sequence_number(sequence_number as u32)
                        .version(version as u32))
                })
                .boxed())
        };

        Box::pin(fut)
    }

    fn remove(&mut self, _id: Self::SourceId) -> BoxFuture<StoreResult<()>> {
        unimplemented!()
    }
}

/// Result returning the crate [`SubscriberError`] type.
///
/// [`SubscriberError`]: enum.Error.html
pub type SubscriberResult<T> = Result<T, SubscriberError>;

/// Error types returned by the [`eventually::EventSubscriber`] implementation
/// on the [`EventSubscriber`] type.
///
/// [`eventually::EventSubscriber`]: ../eventually/trait.EventSubscriber.html
/// [`EventSubscriber`]: struct.EventSubscriber.html
#[derive(Debug, thiserror::Error)]
pub enum SubscriberError {
    /// Error returned when failed to establish a [`PubSub`] connection
    /// with Redis.
    ///
    /// [`PubSub`]: https://docs.rs/redis/0.17.0/redis/aio/struct.PubSub.html
    #[error("failed to establish connection with Redis: {0}")]
    Connection(#[source] RedisError),

    /// Error returned when failed to get the payload from a `SUBSCRIBE` event.
    #[error("failed to get payload from message: {0}")]
    Payload(#[source] RedisError),

    /// Error returned when failed to decode the payload received
    /// from JSON.
    #[error("failed to decode published message: {0}")]
    DecodeMessage(#[source] serde_json::Error),

    /// Error returned when failed to execute the `SUBSCRIBE` command
    /// to receive notification on the stream topic.
    #[error("failed to subscriber to stream events: {0}")]
    Subscribe(#[source] RedisError),

    /// Error returned when attempting to decode the source id from the
    /// notification payload.
    #[error("failed to decode source_id from published message: {0}")]
    DecodeSourceId(#[source] anyhow::Error),
}

/// Redis backend implementation for [`eventually::EventSubscriber`] trait.
///
/// [`eventually::EventSubscriber`]: ../eventually/trait.EventSubscriber.html
pub struct EventSubscriber<Id, Event> {
    stream_name: &'static str,
    client: redis::Client,
    id: std::marker::PhantomData<Id>,
    event: std::marker::PhantomData<Event>,
}

impl<Id, Event> eventually_core::subscription::EventSubscriber for EventSubscriber<Id, Event>
where
    Id: TryFrom<String> + Eq + Send + Sync,
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    Event: Send + Sync,
    for<'de> Event: Deserialize<'de>,
{
    type SourceId = Id;
    type Event = Event;
    type Error = SubscriberError;

    fn subscribe_all(&self) -> BoxFuture<SubscriberResult<SubscriberEventStream<Self>>> {
        #[derive(Deserialize)]
        struct SubscribeMessage<Event> {
            source_id: String,
            sequence_number: u32,
            version: u32,
            event: Event,
        }

        let fut = async move {
            let mut pubsub = self
                .client
                .get_async_connection()
                .await
                .map_err(SubscriberError::Connection)?
                .into_pubsub();

            pubsub
                .subscribe(self.stream_name)
                .await
                .map_err(SubscriberError::Subscribe)?;

            Ok(pubsub
                .into_on_message()
                .map(|msg| msg.get_payload::<Vec<u8>>())
                .map_err(SubscriberError::Payload)
                .and_then(|payload| async move {
                    let msg: SubscribeMessage<Event> =
                        serde_json::from_slice(&payload).map_err(SubscriberError::DecodeMessage)?;

                    let source_id = Id::try_from(msg.source_id)
                        .map_err(anyhow::Error::from)
                        .map_err(SubscriberError::DecodeSourceId)?;

                    Ok(Persisted::from(source_id, msg.event)
                        .sequence_number(msg.sequence_number)
                        .version(msg.version))
                })
                .boxed())
        };

        Box::pin(fut)
    }
}

/// [`futures::Stream`] implementation for Redis `XRANGE` operations.
///
/// [`futures::Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
struct RedisPaginatedStream {
    conn: redis::aio::MultiplexedConnection,
    stream_name: String,
    page_size: usize,
    from: usize,
}

impl RedisPaginatedStream {
    /// Returns a [`futures::Stream`] instance out of the paginated stream instance.
    ///
    /// This implementation will fetch all requested entries from a Redis Stream
    /// using pagination.
    ///
    /// Each page is as big as `page_size`; for each page requested,
    /// all the entries in [`StreamRangeReply`] are yielded in the stream,
    /// until the entries are fully exhausted.
    ///
    /// The stream stop when all entries in the Redis Stream have been returned.
    ///
    /// [`futures::Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
    /// [`StreamRangeReply`]: https://docs.rs/redis/0.17.0/redis/streams/struct.StreamRangeReply.html
    fn into_stream(mut self) -> impl Stream<Item = RedisResult<StreamId>> + 'static {
        async_stream::try_stream! {
            let mut from = self.from;

            loop {
                let result: StreamRangeReply = self.conn.xrange_count(&self.stream_name, from, "+", self.page_size).await?;
                let ids = result.ids;
                let size = ids.len();

                for id in ids {
                    from = parse_entry_id(&id.id).0 + 1;
                    yield id;
                }

                if size < self.page_size {
                    break;
                }
            }
        }
    }
}

fn parse_entry_id(id: &str) -> (usize, usize) {
    let parts: Vec<&str> = id.split("-").collect();
    (parts[0].parse().unwrap(), parts[1].parse().unwrap())
}
