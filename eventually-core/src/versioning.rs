//! Contains support for Optimistic Concurrency Control through
//! Versioning.

/// Data type that carries a version for Optimistic Concurrency Control.
pub trait Versioned {
    /// Current version of the data.
    fn version(&self) -> i64;
}

impl<T> Versioned for Option<T>
where
    T: Versioned,
{
    #[inline]
    fn version(&self) -> i64 {
        self.as_ref().map(Versioned::version).unwrap_or_default()
    }
}
