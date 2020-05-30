//! Container for the fundamental types and abstraction to support Event Sourcing
//! and Command-Query Responsibility Segregation patterns.
//!
//! ## Note
//!
//! Generally, you should not depend from this crate directly, but instead
//! import the public crate: [`eventually`].
//!
//! Please refer to the public crate API documentation for more information.
//!
//! [`eventually`]: https://crates.io/crates/eventually

pub mod aggregate;
pub mod repository;
pub mod store;
pub mod versioning;
