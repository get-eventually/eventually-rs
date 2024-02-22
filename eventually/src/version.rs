//! Contains the types necessary for Optimistic Locking through versioning.

/// A version used for Optimistic Locking.
///
/// Used by the [crate::aggregate::Root] to avoid concurrency issues,
/// and [crate::event::Store] to implement stream-local ordering to the messages.
pub type Version = u64;

/// Used to set a specific expectation during an operation
/// that mutates some sort of resource (e.g. an [Event Stream][crate::event::Stream])
/// that supports versioning.
///
/// It allows for optimistic locking, avoiding data races
/// when modifying the same resource at the same time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// Disables any kind of optimistic locking check, allowing
    /// for any [Version] to be used compared to the new one.
    Any,
    /// Expects that the previous [Version] used for the operation
    /// must have the value specified.
    MustBe(Version),
}

/// This error is returned by a function when a version conflict error has
/// been detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("conflict error detected, expected version was: {expected}, found: {actual}")]
pub struct ConflictError {
    /// The [Version] value that was expected when calling the function that failed.
    pub expected: Version,

    /// The actual [Version] value, which mismatch caused this error.
    pub actual: Version,
}
