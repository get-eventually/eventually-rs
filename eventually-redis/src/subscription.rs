#![allow(missing_docs)]

use std::convert::TryFrom;
use std::fmt::Debug;

use eventually::store::Persisted;
use eventually::subscription::{Subscription, SubscriptionStream};

use futures::future::BoxFuture;
use futures::stream::{StreamExt, TryStreamExt};

use redis::streams::StreamKey;
use redis::{AsyncCommands, RedisError, RedisResult};

use serde::Deserialize;

use crate::stream;

/// Result returning the crate [`SubscriptionError`] type.
///
/// [`SubscriptionError`]: enum.Error.html
pub type SubscriptionResult<T> = Result<T, SubscriptionError>;

/// Error types returned by the [`eventually::Subscription`] implementation
/// on the [`PersistentSubscription`] type.
///
/// [`eventually::Subscription`]: ../eventually/trait.Subscription.html
/// [`PersistentSubscription`]: struct.PersistentSubscription.html
#[derive(Debug, thiserror::Error)]
pub enum SubscriptionError {
    /// Error returned when failed to decoding events from JSON
    /// from the `XREADGROUP` operation.
    #[error("failed to decode events: {0}")]
    DecodeEvents(#[source] serde_json::Error),

    /// Error returned when reading the stream coming from the `XREADGROUP`
    /// operation.
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

    /// Error returned when failed to acknowledge one Redis message
    /// using `XACK` command, due to an error occurred on Redis server.
    #[error("failed to checkpoint subscription due to Redis error: version {0}, {1}")]
    CheckpointFromRedis(u32, #[source] RedisError),

    /// Error returned when Redis didn't acknowledge an `XACK` command,
    /// likely due to an incorrect version number provided.
    #[error("checkpoint subscription not acknowledged by Redis, check the version: version {0}")]
    Checkpoint(u32),
}

#[derive(Clone)]
pub struct PersistentSubscription<Id, Event> {
    pub(crate) stream: &'static str,
    pub(crate) group_name: &'static str,
    pub(crate) conn: redis::aio::MultiplexedConnection,
    pub(crate) stream_page_size: usize,
    pub(crate) id: std::marker::PhantomData<Id>,
    pub(crate) event: std::marker::PhantomData<Event>,
}

impl<Id, Event> PersistentSubscription<Id, Event> {
    /// Creates the Consumer Group in order to use `XREADGROUP` during
    /// [`resume`] processing.
    ///
    /// [`resume`]: struct.PersistentSubscription.html#method.resume
    pub(crate) async fn create_consumer_group(&mut self) -> RedisResult<()> {
        let result: RedisResult<()> = self
            .conn
            .xgroup_create_mkstream(self.stream, self.group_name, 0)
            .await;

        if let Err(ref err) = result {
            if let Some("BUSYGROUP") = err.code() {
                // Consumer group has already been created, skip error.
                return Ok(());
            }
        }

        result
    }
}

impl<Id, Event> Subscription for PersistentSubscription<Id, Event>
where
    Id: TryFrom<String> + Debug + Eq + Clone + Send + Sync,
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    Event: Debug + Send + Sync,
    for<'de> Event: Deserialize<'de>,
{
    type SourceId = Id;
    type Event = Event;
    type Error = SubscriptionError;

    fn resume(&self) -> BoxFuture<SubscriptionResult<SubscriptionStream<Self>>> {
        let fut = async move {
            let keys_stream = stream::into_xread_stream(
                self.conn.clone(),
                self.stream.to_owned(),
                self.group_name.to_owned(),
                self.stream_page_size,
            );

            Ok(keys_stream
                .map_err(SubscriptionError::Stream)
                .and_then(|StreamKey { ids, .. }| async move {
                    Ok(futures::stream::iter(ids.into_iter().map(Ok)))
                })
                .try_flatten()
                // TODO: merge this in its own function with EventStore.stream_all
                .and_then(|entry| async move {
                    let source_id: String = entry
                        .get("source_id")
                        .ok_or_else(|| SubscriptionError::NoKey("source_id"))?;

                    let source_id: Id = Id::try_from(source_id)
                        .map_err(anyhow::Error::from)
                        .map_err(SubscriptionError::DecodeSourceId)?;

                    let event: Vec<u8> = entry
                        .get("event")
                        .ok_or_else(|| SubscriptionError::NoKey("event"))?;

                    let event: Event =
                        serde_json::from_slice(&event).map_err(SubscriptionError::DecodeEvents)?;

                    let version: u32 = entry
                        .get("version")
                        .ok_or_else(|| SubscriptionError::NoKey("version"))?;

                    let sequence_number = stream::parse_version(&entry.id);

                    Ok(Persisted::from(source_id, event)
                        .sequence_number(sequence_number as u32)
                        .version(version))
                })
                .boxed())
        };

        Box::pin(fut)
    }

    fn checkpoint(&self, version: u32) -> BoxFuture<SubscriptionResult<()>> {
        let fut = async move {
            let mut conn = self.conn.clone();
            let stream_version = format!("{}-1", version);

            let ok: bool = conn
                .xack(self.stream, self.group_name, &[&stream_version])
                .await
                .map_err(|e| SubscriptionError::CheckpointFromRedis(version, e))?;

            if !ok {
                return Err(SubscriptionError::Checkpoint(version));
            }

            Ok(())
        };

        Box::pin(fut)
    }
}
