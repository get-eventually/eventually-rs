//! Collection of utilities that extends or implements some of the traits
//! found in [`eventually-core`].
//!
//! [`eventually-core`]: https://crates.io/crates/eventually-core

pub mod inmemory;
pub mod optional;
pub mod sync;

pub use tokio::spawn;
