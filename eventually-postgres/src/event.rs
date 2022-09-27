use std::{marker::PhantomData, string::ToString};

use async_trait::async_trait;
use chrono::Utc;
use eventually::{
    event,
    message::{Message, Metadata},
    serde::{Deserializer, Serde},
    version,
    version::Version,
};
use futures::{future::ready, StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::{
    postgres::{PgDatabaseError, PgRow},
    PgPool, Postgres, Row, Transaction,
};

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

#[derive(Debug, thiserror::Error)]
pub enum AppendError {
    #[error("conflict error detected: {0})")]
    Conflict(#[source] version::ConflictError),
    #[error("failed to begin transaction: {0}")]
    BeginTransaction(#[source] sqlx::Error),
    #[error("failed to upsert new event stream version")]
    UpsertEventStream,
    #[error("failed to append a new domain event: {0}")]
    AppendEvent(#[source] sqlx::Error),
    #[error("failed to commit transaction: {0}")]
    CommitTransaction(#[source] sqlx::Error),
    #[error("db returned an error: {0}")]
    Database(#[from] sqlx::Error),
}

impl From<AppendError> for Option<version::ConflictError> {
    fn from(err: AppendError) -> Self {
        match err {
            AppendError::Conflict(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Store<Id, Evt, OutEvt, S>
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

impl<Id, Evt, OutEvt, S> Store<Id, Evt, OutEvt, S>
where
    Id: ToString + Clone,
    Evt: TryFrom<OutEvt>,
    OutEvt: From<Evt>,
    S: Serde<OutEvt>,
{
    pub async fn new(pool: PgPool, serde: S) -> Result<Self, sqlx::migrate::MigrateError> {
        // Make sure the latest migrations are used before using the Store instance.
        crate::MIGRATIONS.run(&pool).await?;

        Ok(Self {
            pool,
            serde,
            id_type: PhantomData,
            evt_type: PhantomData,
            out_evt_type: PhantomData,
        })
    }
}

lazy_static! {
    static ref CONFLICT_ERROR_REGEX: Regex = Regex::new(
        r#"event stream version check failed, expected: (?P<expected>\d), got: (?P<got>\d)"#
    )
    .expect("regex compiles successfully");
}

fn check_for_conflict_error(err: sqlx::Error) -> AppendError {
    fn capture_to_version(captures: &regex::Captures, name: &'static str) -> Version {
        let v: i32 = captures
            .name(name)
            .expect("field is captured")
            .as_str()
            .parse::<i32>()
            .expect("field should be a valid integer");

        v as Version
    }

    match err {
        sqlx::Error::Database(ref pg_err) => {
            match CONFLICT_ERROR_REGEX.captures(pg_err.message()) {
                None => AppendError::Database(err),
                Some(captures) => AppendError::Conflict(version::ConflictError {
                    actual: capture_to_version(&captures, "got"),
                    expected: capture_to_version(&captures, "expected"),
                }),
            }
        }
        _ => AppendError::Database(err),
    }
}

fn try_get_column<T>(row: &PgRow, name: &'static str) -> Result<T, StreamError>
where
    for<'a> T: sqlx::Type<Postgres> + sqlx::Decode<'a, Postgres>,
{
    row.try_get(name)
        .map_err(|err| StreamError::ReadColumn { name, error: err })
}

impl<Id, Evt, OutEvt, S> Store<Id, Evt, OutEvt, S>
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
        let version_column: i32 = try_get_column(&row, "version")?;
        let event_column: Vec<u8> = try_get_column(&row, "event")?;
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

    async fn append_domain_event(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event_stream_id: &str,
        event_version: i32,
        new_event_stream_version: i32,
        event: event::Envelope<Evt>,
    ) -> Result<(), AppendError> {
        let event_type = event.message.name();
        let out_event = OutEvt::from(event.message);
        let serialized_event = self.serde.serialize(out_event);
        let mut metadata = event.metadata;

        metadata.insert("Recorded-At".to_owned(), Utc::now().to_rfc3339());
        metadata.insert(
            "Recorded-With-New-Version".to_owned(),
            new_event_stream_version.to_string(),
        );

        sqlx::query(
            r#"INSERT INTO events (event_stream_id, "type", "version", event, metadata) VALUES ($1, $2, $3, $4, $5)"#,
        )
            .bind(event_stream_id)
            .bind(event_type)
            .bind(event_version)
            .bind(serialized_event)
            .bind(sqlx::types::Json(metadata))
            .execute(tx)
            .await
            .map_err(AppendError::AppendEvent)?;

        Ok(())
    }

    async fn append_domain_events(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event_stream_id: &str,
        new_version: i32,
        events: Vec<event::Envelope<Evt>>,
    ) -> Result<(), AppendError> {
        let current_event_stream_version = new_version - (events.len() as i32);

        for (i, event) in events.into_iter().enumerate() {
            let event_version = current_event_stream_version + (i as i32) + 1;

            self.append_domain_event(tx, event_stream_id, event_version, new_version, event)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<Id, Evt, OutEvt, S> event::Store for Store<Id, Evt, OutEvt, S>
where
    Id: ToString + Clone + Send + Sync,
    Evt: TryFrom<OutEvt> + Message + std::fmt::Debug + Send + Sync,
    <Evt as TryFrom<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
    OutEvt: From<Evt> + Send + Sync,
    S: Serde<OutEvt> + Send + Sync,
    <S as Deserializer<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
{
    type StreamId = Id;
    type Event = Evt;
    type StreamError = StreamError;
    type AppendError = AppendError;

    fn stream(
        &self,
        id: &Self::StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<Self::StreamId, Self::Event, Self::StreamError> {
        let from_version: i32 = match select {
            event::VersionSelect::All => 0,
            event::VersionSelect::From(v) => v as i32,
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(AppendError::BeginTransaction)?;

        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE DEFERRABLE")
            .execute(&mut tx)
            .await?;

        let string_id = id.to_string();

        let new_version: i32 = match version_check {
            event::StreamVersionExpected::Any => {
                let events_len = events.len() as i32;

                sqlx::query("SELECT * FROM upsert_event_stream_with_no_version_check($1, $2)")
                    .bind(&string_id)
                    .bind(events_len)
                    .fetch_one(&mut tx)
                    .await
                    .and_then(|row| row.try_get(0))?
            }
            event::StreamVersionExpected::MustBe(v) => {
                let new_version = v + (events.len() as Version);

                sqlx::query("CALL upsert_event_stream($1, $2, $3)")
                    .bind(&string_id)
                    .bind(v as i32)
                    .bind(new_version as i32)
                    .execute(&mut tx)
                    .await
                    .map_err(check_for_conflict_error)
                    .map(|_| new_version as i32)?
            }
        };

        self.append_domain_events(&mut tx, &string_id, new_version, events)
            .await?;

        tx.commit().await.map_err(AppendError::CommitTransaction)?;

        Ok(new_version as Version)
    }
}
