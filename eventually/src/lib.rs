#![allow(missing_docs)]

pub mod aggregate;
pub mod command;
pub mod eventstore;
pub mod inmemory;
pub mod scenario;
pub mod subscription;

use std::collections::HashMap;

use serde::Serialize;

pub type Metadata = HashMap<String, MetadataValue>;

pub type Events<T> = Vec<Event<T>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Event<T> {
    pub payload: T,
    pub metadata: Metadata,
}

impl<T> From<T> for Event<T> {
    #[inline]
    fn from(payload: T) -> Self {
        Self {
            payload,
            metadata: HashMap::new(),
        }
    }
}

impl<T> Event<T> {
    pub fn with_string_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, MetadataValue::String(value));
        self
    }

    pub fn with_i64_metadata(mut self, key: String, value: i64) -> Self {
        self.metadata.insert(key, MetadataValue::Integer(value));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum MetadataValue {
    String(String),
    Integer(i64),
}

impl Into<Option<i64>> for MetadataValue {
    #[inline]
    fn into(self) -> Option<i64> {
        match self {
            MetadataValue::Integer(v) => Some(v),
            _ => None,
        }
    }
}

impl Into<Option<String>> for MetadataValue {
    #[inline]
    fn into(self) -> Option<String> {
        match self {
            MetadataValue::String(v) => Some(v),
            _ => None,
        }
    }
}
