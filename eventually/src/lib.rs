#![allow(type_alias_bounds)]
#![warn(missing_docs)]

pub mod aggregate;
pub mod command;
pub mod optional;
pub mod store;
pub mod versioned;

pub use {
    aggregate::{Aggregate, AggregateExt},
    command::Handler as CommandHandler,
    store::Store,
};
