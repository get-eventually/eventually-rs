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

mod store;
mod subscriber;

pub use store::*;
pub use subscriber::*;
