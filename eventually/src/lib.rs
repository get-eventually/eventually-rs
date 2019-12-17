#![allow(type_alias_bounds)]

pub mod aggregate;
pub mod command;
pub mod store;
pub mod versioned;

pub use {
    aggregate::{Aggregate, AggregateExt},
    command::Handler as CommandHandler,
    store::{ReadStore, WriteStore},
};
