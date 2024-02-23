//! `eventually` is a crate that helps you apply different patterns to your Rust
//! application domain code, such as: Event Sourcing, Aggregate Root, Outbox Pattern,
//! and so on.

#![deny(unsafe_code, unused_qualifications, trivial_casts, missing_docs)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

pub mod aggregate;
pub mod command;
pub mod event;
pub mod message;
// pub mod query;
pub mod serde;
#[cfg(feature = "tracing")]
pub mod tracing;
pub mod version;
