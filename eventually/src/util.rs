//! Collection of utilities that extends or implements some of the traits
//! found in this crate.

#[deny(
    clippy::all,
    missing_docs,
    unsafe_code,
    unused_qualifications,
    trivial_casts
)]
pub mod inmemory;
pub mod optional;
pub mod sync;

pub use tokio::spawn;
