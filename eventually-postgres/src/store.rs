use std::{marker::PhantomData, string::ToString};

use async_trait::async_trait;
use eventually::{
    event,
    message::{Message, Metadata},
    version,
    version::Version,
};
use futures::{future::ready, StreamExt, TryStreamExt};
use sqlx::{postgres::PgRow, PgPool, Postgres, Row};

use crate::serde::{ByteArray, Deserializer, Serde};

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("failed to convert domain event from its serialization type: {0}")]
    ConvertEvent(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("failed to deserialize event from database: {0}")]
    DeserializeEvent(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("failed to get column '{name}' from result row: {error}")]
    ReadColumn {
        name: &'static str,
        #[source]
        error: sqlx::Error,
    },
    #[error("db returned an error: {0}")]
    Database(#[source] sqlx::Error),
}

#[derive(Debug, Clone)]
pub struct EventStore<Id, Evt, OutEvt, S>
where
    Id: ToString + Clone,
    Evt: TryFrom<OutEvt>,
    OutEvt: From<Evt>,
    S: Serde<OutEvt>,
{
    pool: PgPool,
    serde: S,
    id_type: PhantomData<Id>,
    evt_type: PhantomData<Evt>,
    out_evt_type: PhantomData<OutEvt>,
}

fn try_get_column<T>(row: &PgRow, name: &'static str) -> Result<T, StreamError>
where
    for<'a> T: sqlx::Type<Postgres> + sqlx::Decode<'a, Postgres>,
{
    row.try_get(name)
        .map_err(|err| StreamError::ReadColumn { name, error: err })
}

impl<Id, Evt, OutEvt, S> EventStore<Id, Evt, OutEvt, S>
where
    Id: ToString + Clone + Send + Sync,
    Evt: TryFrom<OutEvt> + Message + Send + Sync,
    <Evt as TryFrom<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
    OutEvt: From<Evt> + Send + Sync,
    S: Serde<OutEvt> + Send + Sync,
    <S as Deserializer<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
{
    fn event_row_to_persisted_event(
        &self,
        stream_id: Id,
        row: PgRow,
    ) -> Result<event::Persisted<Id, Evt>, StreamError> {
        let version_column: i64 = try_get_column(&row, "version")?;
        let event_column: ByteArray = try_get_column(&row, "event")?;
        let metadata_column: sqlx::types::Json<Metadata> = try_get_column(&row, "metadata")?;

        let deserialized_event = self
            .serde
            .deserialize(event_column)
            .map_err(|err| StreamError::DeserializeEvent(Box::new(err)))?;

        let converted_event = Evt::try_from(deserialized_event)
            .map_err(|err| StreamError::ConvertEvent(Box::new(err)))?;

        Ok(event::Persisted {
            stream_id,
            version: version_column as Version,
            event: event::Envelope {
                message: converted_event,
                metadata: metadata_column.0,
            },
        })
    }
}

#[async_trait]
impl<Id, Evt, OutEvt, S> event::Store for EventStore<Id, Evt, OutEvt, S>
where
    Id: ToString + Clone + Send + Sync,
    Evt: TryFrom<OutEvt> + Message + Send + Sync,
    <Evt as TryFrom<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
    OutEvt: From<Evt> + Send + Sync,
    S: Serde<OutEvt> + Send + Sync,
    <S as Deserializer<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
{
    type StreamId = Id;
    type Event = Evt;
    type StreamError = StreamError;
    type AppendError = Option<version::ConflictError>;

    fn stream(
        &self,
        id: &Self::StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<Self::StreamId, Self::Event, Self::StreamError> {
        let from_version: i64 = match select {
            event::VersionSelect::All => 0,
            event::VersionSelect::From(v) => v as i64,
        };

        let query = sqlx::query(
            r#"SELECT version, event, metadata
               FROM events
               WHERE event_stream_id = $1 AND version >= $2
               ORDER BY version"#,
        );

        let id = id.clone();

        query
            .bind(id.to_string())
            .bind(from_version)
            .fetch(&self.pool)
            .map_err(StreamError::Database)
            .and_then(move |row| ready(self.event_row_to_persisted_event(id.clone(), row)))
            .boxed()
    }

    async fn append(
        &self,
        id: Self::StreamId,
        version_check: event::StreamVersionExpected,
        events: Vec<event::Envelope<Self::Event>>,
    ) -> Result<Version, Self::AppendError> {
        todo!()
    }
}
