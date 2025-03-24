//! `eventually` is a crate that helps you apply different patterns to your Rust
//! application domain code, such as: Event Sourcing, Aggregate Root, Outbox Pattern,
//! and so on.

#![deny(unsafe_code, unused_qualifications, trivial_casts, missing_docs)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

#[cfg(feature = "tracing")]
pub use eventually_core::tracing;
pub use eventually_core::{aggregate, command, event, message, query, serde, version};
#[cfg(feature = "macros")]
pub use eventually_macros::*;
