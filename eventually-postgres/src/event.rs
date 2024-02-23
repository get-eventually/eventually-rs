use std::marker::PhantomData;
use std::string::ToString;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::Utc;
use eventually::message::{Message, Metadata};
use eventually::serde::Serde;
use eventually::version::Version;
use eventually::{event, version};
use futures::future::ready;
use futures::{StreamExt, TryStreamExt};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, Row, Transaction};

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

pub(crate) async fn append_domain_event<Evt, OutEvt>(
    tx: &mut Transaction<'_, Postgres>,
    serde: &impl Serde<OutEvt>,
    event_stream_id: &str,
    event_version: i32,
    new_event_stream_version: i32,
    event: event::Envelope<Evt>,
) -> Result<(), sqlx::Error>
where
    Evt: Message,
    OutEvt: From<Evt>,
{
    let event_type = event.message.name();
    let out_event = OutEvt::from(event.message);
    let serialized_event = serde.serialize(out_event);
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
            .execute(&mut **tx)
            .await?;

    Ok(())
}

pub(crate) async fn append_domain_events<Evt, OutEvt>(
    tx: &mut Transaction<'_, Postgres>,
    serde: &impl Serde<OutEvt>,
    event_stream_id: &str,
    new_version: i32,
    events: Vec<event::Envelope<Evt>>,
) -> Result<(), sqlx::Error>
where
    Evt: Message,
    OutEvt: From<Evt>,
{
    let current_event_stream_version = new_version - (events.len() as i32);

    for (i, event) in events.into_iter().enumerate() {
        let event_version = current_event_stream_version + (i as i32) + 1;

        append_domain_event(
            tx,
            serde,
            event_stream_id,
            event_version,
            new_version,
            event,
        )
        .await?;
    }

    Ok(())
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
    <S as Serde<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
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
}

impl<Id, Evt, OutEvt, S> event::store::Streamer<Id, Evt> for Store<Id, Evt, OutEvt, S>
where
    Id: ToString + Clone + Send + Sync,
    Evt: TryFrom<OutEvt> + Message + std::fmt::Debug + Send + Sync,
    <Evt as TryFrom<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
    OutEvt: From<Evt> + Send + Sync,
    S: Serde<OutEvt> + Send + Sync,
    <S as Serde<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
{
    type Error = StreamError;

    fn stream(&self, id: &Id, select: event::VersionSelect) -> event::Stream<Id, Evt, Self::Error> {
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
}

#[async_trait]
impl<Id, Evt, OutEvt, S> event::store::Appender<Id, Evt> for Store<Id, Evt, OutEvt, S>
where
    Id: ToString + Clone + Send + Sync,
    Evt: TryFrom<OutEvt> + Message + std::fmt::Debug + Send + Sync,
    <Evt as TryFrom<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
    OutEvt: From<Evt> + Send + Sync,
    S: Serde<OutEvt> + Send + Sync,
    <S as Serde<OutEvt>>::Error: std::error::Error + Send + Sync + 'static,
{
    async fn append(
        &self,
        id: Id,
        version_check: version::Check,
        events: Vec<event::Envelope<Evt>>,
    ) -> Result<Version, event::store::AppendError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| anyhow!("failed to begin transaction: {}", err))?;

        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE DEFERRABLE")
            .execute(&mut *tx)
            .await
            .map_err(|err| anyhow!("failed to begin transaction: {}", err))?;

        let string_id = id.to_string();

        let new_version: i32 = match version_check {
            version::Check::Any => {
                let events_len = events.len() as i32;

                sqlx::query("SELECT * FROM upsert_event_stream_with_no_version_check($1, $2)")
                    .bind(&string_id)
                    .bind(events_len)
                    .fetch_one(&mut *tx)
                    .await
                    .and_then(|row| row.try_get(0))
                    .map_err(|err| anyhow!("failed to upsert new event stream version: {}", err))?
            },
            version::Check::MustBe(v) => {
                let new_version = v + (events.len() as Version);

                sqlx::query("CALL upsert_event_stream($1, $2, $3)")
                    .bind(&string_id)
                    .bind(v as i32)
                    .bind(new_version as i32)
                    .execute(&mut *tx)
                    .await
                    .map_err(|err| match crate::check_for_conflict_error(&err) {
                        Some(err) => event::store::AppendError::Conflict(err),
                        None => match err.as_database_error().and_then(|err| err.code()) {
                            Some(code) if code == "40001" => {
                                event::store::AppendError::Conflict(version::ConflictError {
                                    expected: v,
                                    actual: new_version,
                                })
                            },
                            _ => event::store::AppendError::Internal(anyhow!(
                                "failed to upsert new event stream version: {}",
                                err
                            )),
                        },
                    })
                    .map(|_| new_version as i32)?
            },
        };

        append_domain_events(&mut tx, &self.serde, &string_id, new_version, events)
            .await
            .map_err(|err| anyhow!("failed to append new domain events: {}", err))?;

        tx.commit()
            .await
            .map_err(|err| anyhow!("failed to commit transaction: {}", err))?;

        Ok(new_version as Version)
    }
}
