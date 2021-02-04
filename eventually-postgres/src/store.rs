//! Contains an [`EventStore`] implementation using PostgreSQL
//! as a backend data store.
//!
//! [`EventStore`]: ../../eventually-core/store/trait.EventStore.html

use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::ops::DerefMut;

use eventually_core::aggregate::{Aggregate, AggregateId};
use eventually_core::store::{AppendError, EventStream, Expected, Persisted, Select};

use futures::future::BoxFuture;
use futures::stream::{StreamExt, TryStreamExt};

use serde::{Deserialize, Serialize};

use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::Socket;

use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;

#[cfg(feature = "with-tracing")]
use tracing_futures::Instrument;

use crate::{slice_iter, Params};

/// Embedded migrations module.
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/migrations");
}

/// Result returning the crate [`Error`] type.
///
/// [`Error`]: enum.Error.html
pub type Result<T> = std::result::Result<T, Error>;

/// Result returning the connection pool [`Error`] type.
///
/// [`Error`]: enum.Error.html
pub type PoolResult<T> = std::result::Result<T, RunError<tokio_postgres::Error>>;

/// Error type returned by the [`EventStore`] implementation, which is
/// a _newtype_ wrapper around `tokio_postgres::Error`.
///
/// [`EventStore`]: struct.EventStore.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error when decoding persistent events from the database
    /// back into the application during a [`stream`]
    /// or [`stream_all`] operation.
    ///
    /// [`stream`]: struct.EventStore.html#method.stream
    /// [`stream_all`]: struct.EventStore.html#method.stream_all
    #[error("store failed to decode event from the database: {0}")]
    DecodeEvent(#[source] anyhow::Error),

    /// Error when encoding the events in [`append`] to JSON prior
    /// to sending them to the database.
    ///
    /// [`append`]: struct.EventStore.html#method.append
    #[error("store failed to encode events in json: ${0}")]
    EncodeEvents(#[source] serde_json::Error),

    /// Error returned by Postgres when executing queries.
    #[error("postgres client returned an error: ${0}")]
    Postgres(#[from] tokio_postgres::Error),

    /// Error returned by bb8 connection pool.
    #[error("bb8 connnection pool returned an error: ${0}")]
    Bb8(#[from] RunError<tokio_postgres::Error>),
}

impl AppendError for Error {
    #[inline]
    fn is_conflict_error(&self) -> bool {
        // TODO: implement this
        false
    }
}

const APPEND: &str = "SELECT * FROM append_to_store($1::text, $2::text, $3, $4, $5)";

const CREATE_AGGREGATE_TYPE: &str = "SELECT * FROM create_aggregate_type($1::text)";

const STREAM: &str = "SELECT e.*
    FROM events e LEFT JOIN aggregates a
        ON a.id = e.aggregate_id AND a.aggregate_type_id = e.aggregate_type
    WHERE a.aggregate_type_id = $1 AND e.aggregate_id = $2 AND e.version >= $3
    ORDER BY version ASC";

const STREAM_ALL: &str = "SELECT e.*
    FROM events e LEFT JOIN aggregates a
         ON a.id = e.aggregate_id AND a.aggregate_type_id = e.aggregate_type
    WHERE a.aggregate_type_id = $1 AND e.sequence_number >= $2
    ORDER BY e.sequence_number ASC";

const REMOVE: &str = "DELETE FROM aggregates WHERE aggregate_type_id = $1 AND id = $2";

/// Builder type for [`EventStore`] instances.
///
/// [`EventStore`]: struct.EventStore.html
pub struct EventStoreBuilder {
    #[allow(dead_code)]
    inner: (),
}

impl EventStoreBuilder {
    /// Ensure the database is migrated to the latest version.
    #[cfg_attr(
        feature = "with-tracing",
        tracing::instrument(
            err,
            level = "debug",
            name = "EventStoreBuilder::migrate_database",
            skip(pool)
        )
    )]
    pub async fn migrate_database<Tls>(
        pool: Pool<PostgresConnectionManager<Tls>>,
    ) -> anyhow::Result<Self>
    where
        Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
        <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
        <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
        <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        let mut connection = pool.get().await?;
        embedded::migrations::runner()
            .run_async(connection.deref_mut())
            .await?;

        Ok(Self { inner: () })
    }

    /// Returns a new builder instance after migrations have been completed.
    pub fn builder<Tls>(
        self,
        pool: Pool<PostgresConnectionManager<Tls>>,
    ) -> EventStoreBuilderMigrated<Tls>
    where
        Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
        <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
        <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
        <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        EventStoreBuilderMigrated { pool }
    }
}

/// Builder step for [`EventStore`] instances,
/// after the database migration executed from [`EventStoreBuilder`]
/// has been completed.
///
/// [`EventStore`]: struct.EventStore.html
/// [`EventStoreBuilder`]: struct.EventStoreBuilder.html
pub struct EventStoreBuilderMigrated<Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    pool: Pool<PostgresConnectionManager<Tls>>,
}

impl<Tls> EventStoreBuilderMigrated<Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    /// Creates a new [`EventStore`] instance using the specified name
    /// to identify the source/aggregate type.
    ///
    /// Make sure the name is **unique** in your application.
    ///
    /// [`EventStore`]: struct.EventStore.html
    #[inline]
    pub async fn build<Id, Event, Name: ToString>(
        &self,
        type_name: Name,
    ) -> PoolResult<EventStore<Id, Event, Tls>> {
        let store = EventStore {
            pool: self.pool.clone(),
            type_name: type_name.to_string(),
            id: std::marker::PhantomData,
            payload: std::marker::PhantomData,
        };

        store.create_aggregate_type().await?;

        Ok(store)
    }

    /// Creates a new [`EventStore`] for an [`Aggregate`] type.
    ///
    /// Check out [`build`] for more information.
    ///
    /// [`EventStore`]: struct.EventStore.html
    /// [`Aggregate`]: ../../eventually_core/aggregate/trait.Aggregate.html
    /// [`build`]: struct.EventStoreBuilderMigrated.html#method.build
    #[inline]
    pub async fn for_aggregate<'a, T, Name: ToString>(
        &'a self,
        type_name: Name,
        _: &'a T,
    ) -> PoolResult<EventStore<AggregateId<T>, T::Event, Tls>>
    where
        T: Aggregate,
    {
        self.build::<AggregateId<T>, T::Event, Name>(type_name)
            .await
    }
}

/// [`EventStore`] implementation using a PostgreSQL backend.
///
/// This implementation uses `tokio-postgres` crate to interface with Postgres.
///
/// Check out [`EventStoreBuilder`] for examples to how initialize new
/// instances of this type.
///
/// ### Considerations on the `Id` type
///
/// The `Id` type supplied to the `EventStore` has to be
/// able to `to_string()` (so implement the `std::fmt::Display` trait)
/// and to be parsed from a `String` (so to implement the `std::convert::TryFrom<String>` trait).
///
/// The error for the `TryFrom<String>` conversion has to be a `std::error::Error`,
/// so as to be able to map it into an `anyhow::Error` generic error value.
///
/// [`EventStore`]: ../../eventually_core/store/trait.EventStore.html
/// [`EventStoreBuilder`]: ../../eventually_core/store/trait.EventStoreBuilder.html
#[derive(Debug, Clone)]
pub struct EventStore<Id, Event, Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    pub(crate) type_name: String,
    pool: Pool<PostgresConnectionManager<Tls>>,
    id: std::marker::PhantomData<Id>,
    payload: std::marker::PhantomData<Event>,
}

impl<Id, Event, Tls> eventually_core::store::EventStore for EventStore<Id, Event, Tls>
where
    Id: TryFrom<String> + Display + Eq + Send + Sync,
    // This bound is for the translation into an anyhow::Error.
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    <Id as TryFrom<String>>::Error: Into<anyhow::Error>,
    Event: Serialize + Send + Sync + Debug,
    for<'de> Event: Deserialize<'de>,
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    type SourceId = Id;
    type Event = Event;
    type Error = Error;

    fn append(
        &mut self,
        id: Self::SourceId,
        version: Expected,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<u32>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::append",
            id = %id,
            version = ?version,
            events = ?events
        );

        let fut = async move {
            let serialized = events
                .into_iter()
                .map(serde_json::to_value)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(Error::EncodeEvents)?;

            let (version, check) = match version {
                Expected::Any => (0i32, false),
                Expected::Exact(v) => (v as i32, true),
            };

            let params: Params = &[
                &self.type_name,
                &id.to_string(),
                &version,
                &check,
                &serialized,
            ];

            let client = self.pool.get().await?;
            let row = client.query_one(APPEND, params).await?;

            let id: i32 = row.try_get("version")?;
            Ok(id as u32)
        };

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
    }

    fn stream(&self, id: Self::SourceId, select: Select) -> BoxFuture<Result<EventStream<Self>>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::stream",
            id = %id,
            select = ?select,
        );

        let fut = async move {
            let from = match select {
                Select::All => 0i32,
                Select::From(v) => v as i32,
            };

            let id = id.to_string();

            let params: Params = &[&self.type_name, &id, &from];

            self.stream_query(STREAM, params).await
        };

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
    }

    fn stream_all(&self, select: Select) -> BoxFuture<Result<EventStream<Self>>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::stream_all",
            select = ?select
        );

        let fut = async move {
            let from = match select {
                Select::All => 0i64,
                Select::From(v) => v as i64,
            };

            let params: Params = &[&self.type_name, &from];

            self.stream_query(STREAM_ALL, params).await
        };

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
    }

    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<()>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::remove",
            id = %id,
        );

        let fut = async move {
            let client = self.pool.get().await?;
            Ok(client
                .execute(REMOVE, &[&self.type_name, &id.to_string()])
                .await
                .map(|_| ())?)
        };

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
    }
}

impl<Id, Event, Tls> EventStore<Id, Event, Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    #[cfg_attr(
        feature = "with-tracing",
        tracing::instrument(
            err,
            level = "debug",
            name = "EventStore::create_aggregate_type",
            skip(self)
        )
    )]
    async fn create_aggregate_type(&self) -> PoolResult<()> {
        let params: Params = &[&self.type_name];

        let client = self.pool.get().await?;
        client.execute(CREATE_AGGREGATE_TYPE, params).await?;

        Ok(())
    }
}

impl<Id, Event, Tls> EventStore<Id, Event, Tls>
where
    Id: TryFrom<String> + Display + Eq + Send + Sync,
    // This bound is for the translation into an anyhow::Error.
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    Event: Serialize + Send + Sync + Debug,
    for<'de> Event: Deserialize<'de>,
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    async fn stream_query(&self, query: &str, params: Params<'_>) -> Result<EventStream<'_, Self>> {
        let client = self.pool.get().await?;
        Ok(client
            .query_raw(query, slice_iter(params))
            .await
            .map_err(Error::from)?
            .map_err(Error::from)
            .and_then(|row| async move {
                let event: Event = serde_json::from_value(
                    row.try_get("event")
                        .map_err(anyhow::Error::from)
                        .map_err(Error::DecodeEvent)?,
                )
                .map_err(anyhow::Error::from)
                .map_err(Error::DecodeEvent)?;

                let id: String = row
                    .try_get("aggregate_id")
                    .map_err(anyhow::Error::from)
                    .map_err(Error::DecodeEvent)?;

                let version: i32 = row
                    .try_get("version")
                    .map_err(anyhow::Error::from)
                    .map_err(Error::DecodeEvent)?;

                let sequence_number: i64 = row
                    .try_get("sequence_number")
                    .map_err(anyhow::Error::from)
                    .map_err(Error::DecodeEvent)?;

                let id = Id::try_from(id)
                    .map_err(anyhow::Error::from)
                    .map_err(Error::DecodeEvent)?;

                Ok(Persisted::from(id, event)
                    .version(version as u32)
                    .sequence_number(sequence_number as u32))
            })
            .boxed())
    }
}
