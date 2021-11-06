//! Module containing support for Optimistic Concurrency using
//! Versioning.
//!
//! In the future, it will contain support for conflict resolution
//! caused by concurrent writes scenarios.

/// Data type that carries a version for Optimistic Concurrency Control.
pub trait Versioned {
    /// Current version of the data.
    fn version(&self) -> u32;
}

impl<T> Versioned for Option<T>
where
    T: Versioned,
{
    #[inline]
    fn version(&self) -> u32 {
        self.as_ref().map(Versioned::version).unwrap_or_default()
    }
}
