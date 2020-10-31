use std::convert::TryFrom;
use std::fmt::Debug;

use eventually_core::store::Persisted;
use eventually_core::subscription::EventStream as SubscriberEventStream;

use futures::future::BoxFuture;
use futures::stream::{StreamExt, TryStreamExt};

use redis::RedisError;

use serde::Deserialize;

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
#[derive(Clone)]
pub struct EventSubscriber<Id, Event> {
    pub(crate) stream_name: &'static str,
    pub(crate) client: redis::Client,
    pub(crate) id: std::marker::PhantomData<Id>,
    pub(crate) event: std::marker::PhantomData<Event>,
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
