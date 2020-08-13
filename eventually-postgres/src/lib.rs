//! [`eventually`] type implementations for PostgreSQL.
//!
//! ## Event Store
//!
//! This crate includes an [`EventStore`] implementation using PostgreSQL
//! as backend data source.
//!
//! Example usage:
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use eventually_postgres::EventStoreBuilder;
//! #
//! # async fn dox() -> Result<(), Box<dyn std::error::Error>> {
//! // Open a connection with Postgres.
//! let (mut client, connection) =
//!     tokio_postgres::connect("postgres://user@pass:localhost:5432/db", tokio_postgres::NoTls)
//!         .await
//!         .map_err(|err| {
//!             eprintln!("failed to connect to Postgres: {}", err);
//!             err
//!         })?;
//!
//! // The connection, responsible for the actual IO, must be handled by a different
//! // execution context.
//! tokio::spawn(async move {
//!     if let Err(e) = connection.await {
//!         eprintln!("connection error: {}", e);
//!     }
//! });
//!
//! // A domain event example -- it is deliberately simple.
//! #[derive(Debug, Clone)]
//! struct SomeEvent;
//!
//! // Use an EventStoreBuilder to build multiple EventStore instances.
//! let builder = EventStoreBuilder::migrate_database(&mut client)
//!     .await?
//!     .builder(Arc::new(client));
//!
//! // Event store for the events.
//! //
//! // When building an new EventStore instance, a type name is always needed
//! // to distinguish between different aggregates.
//! //
//! // You can also use std::any::type_name for that.
//! let store = builder.build::<String, SomeEvent>("aggregate-name").await?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! [`eventually`]: https://docs.rs/eventually
//! [`EventStore`]: struct.EventStore.html

use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;

use eventually::store::{AppendError, EventStream, Expected, Persisted, Select};
use eventually::{Aggregate, AggregateId};

use futures::future::BoxFuture;
use futures::stream::{StreamExt, TryStreamExt};

use serde::{Deserialize, Serialize};

use tokio_postgres::types::ToSql;
use tokio_postgres::Client;

/// Embedded migrations module.
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/migrations");
}

/// Result returning the crate [`Error`] type.
///
/// [`Error`]: enum.Error.html
pub type Result<T> = std::result::Result<T, Error>;

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
    FROM events e LEFT JOIN aggregates a ON a.id = e.aggregate_id
    WHERE a.aggregate_type_id = $1 AND e.aggregate_id = $2 AND e.version >= $3
    ORDER BY version ASC";

const STREAM_ALL: &str = "SELECT e.*
    FROM events e LEFT JOIN aggregates a ON a.id = e.aggregate_id
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
    pub async fn migrate_database(client: &mut Client) -> anyhow::Result<Self> {
        embedded::migrations::runner().run_async(client).await?;

        Ok(Self { inner: () })
    }

    /// Returns a new builder instance after migrations have been completed.
    pub fn builder(self, client: Arc<Client>) -> EventStoreBuilderMigrated {
        EventStoreBuilderMigrated { client }
    }
}

/// Builder step for [`EventStore`] instances,
/// after the database migration executed from [`EventStoreBuilder`]
/// has been completed.
///
/// [`EventStore`]: struct.EventStore.html
/// [`EventStoreBuilder`]: struct.EventStoreBuilder.html
pub struct EventStoreBuilderMigrated {
    client: Arc<Client>,
}

impl EventStoreBuilderMigrated {
    /// Creates a new [`EventStore`] instance using the specified name
    /// to identify the source/aggregate type.
    ///
    /// Make sure the name is **unique** in your application.
    ///
    /// [`EventStore`]: struct.EventStore.html
    #[inline]
    pub async fn build<Id, Event>(&self, type_name: &'static str) -> Result<EventStore<Id, Event>> {
        let store = EventStore {
            client: self.client.clone(),
            type_name,
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
    pub async fn for_aggregate<'a, T>(
        &'a self,
        type_name: &'static str,
        _: &'a T,
    ) -> Result<EventStore<AggregateId<T>, T::Event>>
    where
        T: Aggregate,
    {
        self.build::<AggregateId<T>, T::Event>(type_name).await
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
pub struct EventStore<Id, Event> {
    client: Arc<Client>,
    type_name: &'static str,
    id: std::marker::PhantomData<Id>,
    payload: std::marker::PhantomData<Event>,
}

impl<Id, Event> eventually::EventStore for EventStore<Id, Event>
where
    Id: TryFrom<String> + Display + Eq + Send + Sync,
    // This bound is for the translation into an anyhow::Error.
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    <Id as TryFrom<String>>::Error: Into<anyhow::Error>,
    Event: Serialize + Send + Sync,
    for<'de> Event: Deserialize<'de>,
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
        Box::pin(async move {
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

            let row = self.client.query_one(APPEND, params).await?;

            let id: i32 = row.try_get("version")?;
            Ok(id as u32)
        })
    }

    fn stream(&self, id: Self::SourceId, select: Select) -> BoxFuture<Result<EventStream<Self>>> {
        Box::pin(async move {
            let from = match select {
                Select::All => 0i32,
                Select::From(v) => v as i32,
            };

            let id = id.to_string();

            let params: Params = &[&self.type_name, &id, &from];

            self.stream_query(STREAM, params).await
        })
    }

    fn stream_all(&self, select: Select) -> BoxFuture<Result<EventStream<Self>>> {
        Box::pin(async move {
            let from = match select {
                Select::All => 0i64,
                Select::From(v) => v as i64,
            };

            let params: Params = &[&self.type_name, &from];

            self.stream_query(STREAM_ALL, params).await
        })
    }

    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<()>> {
        Box::pin(async move {
            Ok(self
                .client
                .execute(REMOVE, &[&self.type_name, &id.to_string()])
                .await
                .map(|_| ())?)
        })
    }
}

impl<Id, Event> EventStore<Id, Event> {
    async fn create_aggregate_type(&self) -> Result<()> {
        let params: Params = &[&self.type_name];

        self.client.execute(CREATE_AGGREGATE_TYPE, params).await?;

        Ok(())
    }
}

impl<Id, Event> EventStore<Id, Event>
where
    Id: TryFrom<String> + Display + Eq + Send + Sync,
    // This bound is for the translation into an anyhow::Error.
    <Id as TryFrom<String>>::Error: std::error::Error + Send + Sync + 'static,
    Event: Serialize + Send + Sync,
    for<'de> Event: Deserialize<'de>,
{
    async fn stream_query(&self, query: &str, params: Params<'_>) -> Result<EventStream<'_, Self>> {
        Ok(self
            .client
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

type Params<'a> = &'a [&'a (dyn ToSql + Sync)];

#[inline]
#[allow(trivial_casts)]
fn slice_iter<'a>(s: Params<'a>) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}
