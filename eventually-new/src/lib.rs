#![allow(missing_docs)]

use std::collections::HashMap;

pub mod aggregate;
pub mod eventstore;
pub mod inmemory;
pub mod scenario;
#[cfg(test)]
pub mod test;

pub use async_trait::async_trait;

#[cfg(feature = "serde")]
use serde::Serialize;

pub type Events<T> = Vec<Event<T>>;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Event<T> {
    data: T,
    metadata: Option<HashMap<String, String>>,
}

impl<T> AsRef<T> for Event<T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<T> From<T> for Event<T> {
    #[inline]
    fn from(data: T) -> Self {
        Self {
            data,
            metadata: None,
        }
    }
}

impl<T> Event<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self::from(data)
    }

    #[inline]
    pub fn new_with_metadata(data: T, metadata: HashMap<String, String>) -> Self {
        Self {
            data,
            metadata: Some(metadata),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.data
    }

    #[inline]
    pub fn metadata(&self) -> Option<&HashMap<String, String>> {
        self.metadata.as_ref()
    }
}
