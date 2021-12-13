#![deny(unsafe_code, unused_qualifications, trivial_casts)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

pub mod aggregate;
pub mod command;
pub mod event;
pub mod message;
pub mod metadata;
pub mod test;
pub mod version;
