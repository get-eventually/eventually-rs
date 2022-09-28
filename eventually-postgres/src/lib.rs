#![deny(unsafe_code, unused_qualifications, trivial_casts)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]

pub mod aggregate;
pub mod event;

pub static MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
