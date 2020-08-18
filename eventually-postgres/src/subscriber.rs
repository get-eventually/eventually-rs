use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::StreamExt;

use eventually_core::store::Persisted;
use eventually_core::subscription::EventStream;

use serde::Deserialize;

use tokio::sync::broadcast;

use tokio_postgres::AsyncMessage;

const DEFAULT_BROADCAST_CHANNEL_SIZE: usize = 128;

/// Alias type for a `Result` having [`DeserializeError`] as error type.alloc
///
/// [`DeserializeError`]: struct.DeserializeError.html
pub type Result<T> = std::result::Result<T, DeserializeError>;

/// Error returned by the `TryStream` on [`subscribe_all`]
/// when deserializing payloads coming from Postgres' `LISTEN`
/// asynchronous notifications.
///
/// [`subscribe_all`]: struct.EventSubscriber.html#method.subscribe_all
#[derive(Debug, Clone, thiserror::Error)]
#[error("failed to deserialize notification payload from JSON: {message}")]
pub struct DeserializeError {
    message: String,
}

/// Subscriber for listening to new events committed to an [`EventStore`],
/// using Postgres' `LISTEN` functionality.
///
/// [`EventStore`]: ../store/struct.EventStore.html
#[derive(Clone)]
pub struct EventSubscriber<Id, Event> {
    tx: broadcast::Sender<Result<Persisted<Id, Event>>>,
}

impl<Id, Event> EventSubscriber<Id, Event>
where
    Id: TryFrom<String> + Debug + Send + Sync + 'static,
    Event: Debug + Send + Sync + 'static,
    for<'de> Id: Deserialize<'de>,
    for<'de> Event: Deserialize<'de>,
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
{
    /// Opens a new `LISTEN` stream on the database pointed by the specified DSN,
    /// for the specified [`Aggregate`] type by the `type_name` parameter.
    ///
    /// Returns an error if the connection with the Postgres database
    /// could not be established or experienced some issues.
    ///
    /// [`Aggregate`]: ../../eventually-core/aggregate/trait.Aggregate.html
    pub async fn new(
        dsn: &str,
        type_name: &str,
    ) -> std::result::Result<Self, tokio_postgres::Error> {
        let (client, mut connection) = tokio_postgres::connect(dsn, tokio_postgres::NoTls).await?;

        let client = Arc::new(client);
        let client_captured = client.clone();

        let (tx, _) = broadcast::channel(DEFAULT_BROADCAST_CHANNEL_SIZE);
        let tx_captured = tx.clone();

        let mut stream = futures::stream::poll_fn(move |cx| connection.poll_message(cx));

        tokio::spawn(async move {
            while let Some(event) = stream.next().await {
                let event = event.expect("subscriber connection failed");

                if let AsyncMessage::Notification(not) = event {
                    #[allow(unused_must_use)]
                    {
                        tx_captured.send(
                            serde_json::from_str::<NotificationPayload<Event>>(not.payload())
                                .map_err(|e| DeserializeError {
                                    message: e.to_string(),
                                })
                                .and_then(TryInto::try_into),
                        );
                    }
                }
            }

            drop(client_captured);
        });

        client
            .batch_execute(&("LISTEN ".to_owned() + type_name + ";"))
            .await?;

        Ok(Self { tx })
    }
}

impl<Id, Event> eventually_core::subscription::EventSubscriber for EventSubscriber<Id, Event>
where
    Id: Eq + Send + Sync + Clone,
    Event: Send + Sync + Clone,
{
    type SourceId = Id;
    type Event = Event;
    type Error = DeserializeError;

    fn subscribe_all(&self) -> BoxFuture<Result<EventStream<Self>>> {
        Box::pin(async move {
            Ok(self
                .tx
                .subscribe()
                .into_stream()
                .filter_map(|r| async { r.ok() })
                .boxed())
        })
    }
}

/// JSON payload coming from a `NOTIFY` instruction on the Postgres [`EventStore`].
///
/// [`EventStore`]: ../store/struct.EventStore.html
#[derive(Debug, Deserialize)]
struct NotificationPayload<Event> {
    source_id: String,
    version: u32,
    sequence_number: u32,
    event: Event,
}

impl<SourceId, Event> TryFrom<NotificationPayload<Event>> for Persisted<SourceId, Event>
where
    SourceId: TryFrom<String>,
    <SourceId as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
{
    type Error = DeserializeError;

    fn try_from(payload: NotificationPayload<Event>) -> Result<Self> {
        let source_id: SourceId = payload.source_id.try_into().map_err(|e| DeserializeError {
            message: format!("could not deserialize source id from string: {:?}", e),
        })?;

        Ok(Persisted::from(source_id, payload.event)
            .version(payload.version)
            .sequence_number(payload.sequence_number))
    }
}
