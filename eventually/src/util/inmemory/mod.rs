//! Contains supporting entities using an in-memory backend.

mod projector;
mod store;

pub use projector::Projector;
pub use store::{ConflictError, EventStore, EventStoreBuilder, LaggedError};
