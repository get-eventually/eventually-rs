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
//! # use eventually_postgres::EventStoreBuilder;
//! #
//! # async fn dox() -> Result<(), Box<dyn std::error::Error>> {
//! // Open a connection with bb8.
//! let pg_manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
//!     "postgres://user@pass:localhost:5432/db",
//!     tokio_postgres::NoTls,
//! )
//! .map_err(|err| {
//!     eprintln!("Failed configuring a Postgres Connection pool: {}", err);
//!     err
//! })?;
//! let pool = bb8::Pool::builder().build(pg_manager).await?;
//!
//! // A domain event example -- it is deliberately simple.
//! #[derive(Debug, Clone)]
//! struct SomeEvent;
//!
//! // Use an EventStoreBuilder to build multiple EventStore instances.
//! let builder = EventStoreBuilder::migrate_database(pool.clone())
//!     .await?
//!     .builder(pool);
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

#[deny(
    clippy::all,
    missing_docs,
    unsafe_code,
    unused_qualifications,
    trivial_casts
)]
pub mod store;
pub mod subscriber;
pub mod subscription;

pub use store::{EventStore, EventStoreBuilder};
pub use subscriber::EventSubscriber;
pub use subscription::PersistentBuilder;

use tokio_postgres::types::ToSql;

/// Adapter type for parameters compatible with `tokio_postgres::Client`
/// methods.
pub(crate) type Params<'a> = &'a [&'a (dyn ToSql + Sync)];

#[inline]
#[allow(trivial_casts)]
pub(crate) fn slice_iter(s: Params) -> impl ExactSizeIterator<Item = &dyn ToSql> {
    s.iter().map(|s| *s as _)
}
