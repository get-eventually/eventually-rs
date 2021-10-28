//! Contains an [`EventSubscriber`] implementation using PostgreSQL
//! as a backend data store and `NOTIFY`/`LISTEN` functionality
//! to power the [`EventStream`].
//!
//! [`EventSubscriber`]:
//! ../../eventually/subscription/trait.EventSubscriber.html
//! [`EventStream`]: ../../eventually/subscription/type.EventStream.html

use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::sync::Arc;

use futures::stream::StreamExt;

use eventually::store::Persisted;
use eventually::subscription::EventStream;

use serde::Deserialize;

use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::wrappers::BroadcastStream;

use tokio_postgres::AsyncMessage;

const DEFAULT_BROADCAST_CHANNEL_SIZE: usize = 128;

/// Alias type for a `Result` having [`SubscriberError`] as error type.alloc
///
/// [`SubscriberError`]: struct.SubscriberError.html
pub type Result<T> = std::result::Result<T, SubscriberError>;

/// Error returned by the `TryStream` on [`subscribe_all`]
///
/// [`subscribe_all`]: struct.EventSubscriber.html#method.subscribe_all
#[derive(Debug, Clone, thiserror::Error)]
pub enum SubscriberError {
    /// Error variant returned when deserializing payloads coming from Postgres'
    /// `LISTEN` asynchronous notifications.
    #[error("failed to deserialize notification payload from JSON: {0}")]
    Deserialize(String),

    /// Error variant returned when the connection, used for `LISTEN`
    /// asynchronous notifications gets dropped. Currently the subscriber
    /// cannot recover from this error and a new one should be created.
    #[error("postgres connection error: {0}")]
    Connection(String),
}

/// Subscriber for listening to new events committed to an [`EventStore`],
/// using Postgres' `LISTEN` functionality.
///
/// [`EventStore`]: ../store/struct.EventStore.html
#[derive(Clone)]
pub struct EventSubscriber<Id, Event> {
    tx: Sender<Result<Persisted<Id, Event>>>,
}

impl<Id, Event> EventSubscriber<Id, Event>
where
    Id: TryFrom<String> + Debug + Send + Sync + Clone + 'static,
    Event: Debug + Send + Sync + Clone + 'static,
    for<'de> Id: Deserialize<'de>,
    for<'de> Event: Deserialize<'de>,
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
{
    /// Opens a new `LISTEN` stream on the database pointed by the specified
    /// DSN, for the specified [`Aggregate`] type by the `type_name`
    /// parameter.
    ///
    /// Returns an error if the connection with the Postgres database
    /// could not be established or experienced some issues.
    ///
    /// [`Aggregate`]: ../../eventually/aggregate/trait.Aggregate.html
    pub async fn new(
        dsn: &str,
        type_name: &str,
    ) -> std::result::Result<Self, tokio_postgres::Error> {
        let (client, mut connection) = tokio_postgres::connect(dsn, tokio_postgres::NoTls).await?;

        let client = Arc::new(client);
        let client_captured = client.clone();

        let (tx, _rx) = channel(DEFAULT_BROADCAST_CHANNEL_SIZE);
        let tx_captured = tx.clone();

        let mut stream = futures::stream::poll_fn(move |cx| connection.poll_message(cx));

        eventually::util::spawn(async move {
            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => {
                        if let AsyncMessage::Notification(not) = event {
                            #[allow(unused_must_use)]
                            {
                                tx_captured.send(
                                    serde_json::from_str::<NotificationPayload<Event>>(
                                        not.payload(),
                                    )
                                    .map_err(|e| SubscriberError::Deserialize(e.to_string()))
                                    .and_then(TryInto::try_into),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        #[allow(unused_must_use)]
                        {
                            tx_captured.send(Err(SubscriberError::Connection(e.to_string())));
                        }
                        break;
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

impl<Id, Event> eventually::subscription::EventSubscriber for EventSubscriber<Id, Event>
where
    Id: Eq + Send + Sync + Clone + 'static,
    Event: Send + Sync + Clone + 'static,
{
    type SourceId = Id;
    type Event = Event;
    type Error = SubscriberError;

    fn subscribe_all(&self) -> EventStream<Self> {
        BroadcastStream::new(self.tx.subscribe())
            .filter_map(|r| async { r.ok() })
            .boxed()
    }
}

/// JSON payload coming from a `NOTIFY` instruction on the Postgres
/// [`EventStore`].
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
    type Error = SubscriberError;

    fn try_from(payload: NotificationPayload<Event>) -> Result<Self> {
        let source_id: SourceId = payload.source_id.try_into().map_err(|e| {
            SubscriberError::Deserialize(format!(
                "could not deserialize source id from string: {:?}",
                e
            ))
        })?;

        Ok(Persisted::from(source_id, payload.event)
            .version(payload.version)
            .sequence_number(payload.sequence_number))
    }
}
