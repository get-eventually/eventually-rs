//! Contains the types necessary for Optimistic Locking through versioning.

/// A version used for Optimistic Locking.
///
/// Used by the [crate::aggregate::Root] to avoid concurrency issues,
/// and [crate::event::Store] to implement stream-local ordering to the messages.
pub type Version = u64;

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
