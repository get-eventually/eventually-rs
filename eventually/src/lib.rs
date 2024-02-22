#![deny(unsafe_code, unused_qualifications, trivial_casts)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(missing_docs)]

pub mod aggregate;
pub mod command;
pub mod event;
pub mod message;
pub mod query;
pub mod serde;
#[cfg(feature = "tracing")]
pub mod tracing;
pub mod version;
