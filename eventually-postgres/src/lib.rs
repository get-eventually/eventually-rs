//! `eventually-postgres` contains different implementations of traits
//! from the [eventually] crate that are specific for `PostgreSQL` databases.
//!
//! Check out the [`aggregate::Repository`] and [`event::Store`] implementations
//! to know more.

#![deny(unsafe_code, unused_qualifications, trivial_casts)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]
#![warn(missing_docs)]

pub mod aggregate;
pub mod event;

pub(crate) static MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

use std::sync::LazyLock;

use eventually::version::{ConflictError, Version};
use regex::Regex;

static CONFLICT_ERROR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"version check failed, expected: (?P<expected>\d), got: (?P<got>\d)")
        .expect("regex compiles successfully")
});

pub(crate) fn check_for_conflict_error(err: &sqlx::Error) -> Option<ConflictError> {
    fn capture_to_version(captures: &regex::Captures, name: &'static str) -> Version {
        let v: i32 = captures
            .name(name)
            .expect("field is captured")
            .as_str()
            .parse::<i32>()
            .expect("field should be a valid integer");

        #[allow(clippy::cast_sign_loss)]
        {
            v as Version
        }
    }

    if let sqlx::Error::Database(ref pg_err) = err {
        return CONFLICT_ERROR_REGEX
            .captures(pg_err.message())
            .map(|captures| ConflictError {
                actual: capture_to_version(&captures, "got"),
                expected: capture_to_version(&captures, "expected"),
            });
    }

    None
}
