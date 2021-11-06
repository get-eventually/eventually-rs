//! Collection of utilities that extends or implements some of the traits
//! found in this crate.

pub mod inmemory;
pub mod optional;
pub mod sync;

pub use tokio::spawn;
