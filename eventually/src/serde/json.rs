//! Contains the [Serializer] and [Deserializer] compatible
//! implementation using JSON.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::serde::Serde;

/// Implements the [Serializer] and [Deserializer] traits from [crate::serde] module,
/// which uses the [serde] crate to serialize and deserialize a message into JSON.
#[derive(Debug, Clone, Copy)]
pub struct JsonSerde<T>(PhantomData<T>)
where
    T: Serialize + Send + Sync,
    for<'d> T: Deserialize<'d>;

impl<T> Default for JsonSerde<T>
where
    T: Serialize + Send + Sync,
    for<'d> T: Deserialize<'d>,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Serde<T> for JsonSerde<T>
where
    T: Serialize + Send + Sync,
    for<'d> T: Deserialize<'d>,
{
    type Error = serde_json::Error;

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error> {
        serde_json::from_slice(&data)
    }

    fn serialize(&self, value: T) -> Vec<u8> {
        serde_json::to_vec(&value).expect("json serialization should not fail")
    }
}
