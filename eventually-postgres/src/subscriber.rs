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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("failed to deserialize notification payload from JSON: {0}")]
    InvalidMessage(String),

    #[error("failed to connect to the database: {0}")]
    Connection(String),

    #[error("error coming from the database: {0}")]
    Database(String),
}

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
    pub async fn new(dsn: &str, type_name: &str) -> Result<Self> {
        let (client, mut connection) = tokio_postgres::connect(dsn, tokio_postgres::NoTls)
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

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
                                .map_err(|e| Error::InvalidMessage(e.to_string()))
                                .and_then(TryInto::try_into),
                        );
                    }
                }
            }

            drop(client_captured);
        });

        client
            .batch_execute(&("LISTEN ".to_owned() + type_name + ";"))
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

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
    type Error = Error;

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
    type Error = Error;

    fn try_from(payload: NotificationPayload<Event>) -> Result<Self> {
        let source_id: SourceId = payload.source_id.try_into().map_err(|e| {
            Error::InvalidMessage(format!(
                "could not deserialize source id from string: {:?}",
                e
            ))
        })?;

        Ok(Persisted::from(source_id, payload.event)
            .version(payload.version)
            .sequence_number(payload.sequence_number))
    }
}
