//! This module provides traits and implementations for serialization and
//! deserialization, allowing you to convert Rust data structures to and from
//! different formats like JSON, Protobuf, etc.

use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::anyhow;
#[cfg(feature = "serde-prost")]
use prost::bytes::Bytes;
#[cfg(feature = "serde-json")]
use serde::{Deserialize, Serialize};

/// A serializer interface that can be used to serialize a Rust data type
/// into a specific wire format as a byte array.
pub trait Serializer<T>: Send + Sync {
    /// Serializes the given value into the protocol supported by this implementation.
    ///
    /// # Errors
    ///
    /// An error ([`anyhow::Error`]) is returned in case the serialization could not
    /// succeed as expected.
    fn serialize(&self, value: T) -> anyhow::Result<Vec<u8>>;
}

/// A deserializer interface that can be used to deserialize a byte array
/// into an instance of a specific Rust data type from a specific wire format.
pub trait Deserializer<T>: Send + Sync {
    /// Deserializes the given value from a message encoded in the wire format
    /// supported by this implementation.
    ///
    /// # Errors
    ///
    /// An error ([`anyhow::Error`]) is returned in case the deserialization could not
    /// succeed as expected.
    fn deserialize(&self, data: &[u8]) -> anyhow::Result<T>;
}

/// [Serializer] and [Deserializer] that can be used to serialize into and deserialize
/// from a given type into a specific wire format, such as JSON, Protobuf, etc.
pub trait Serde<T>: Serializer<T> + Deserializer<T> + Send + Sync {}

impl<S, T> Serde<T> for S where S: Serializer<T> + Deserializer<T> {}

/// Implements the [Serde] trait to translate between two different types,
/// and using the specified [Serde] for serialization and deserialization
/// using the new `Out` type.
#[derive(Clone)]
pub struct Convert<In, Out, S>
where
    In: Send + Sync,
    Out: Send + Sync,
    S: Serde<Out> + Send + Sync,
{
    serde: S,
    inn: PhantomData<In>,
    out: PhantomData<Out>,
}

impl<In, Out, S> Convert<In, Out, S>
where
    In: Send + Sync,
    Out: Send + Sync,
    S: Serde<Out> + Send + Sync,
{
    /// Creates a new [Convert] serde instance.
    pub fn new(serde: S) -> Self {
        Self {
            serde,
            inn: PhantomData,
            out: PhantomData,
        }
    }
}

impl<In, Out, S> Serializer<In> for Convert<In, Out, S>
where
    In: TryFrom<Out> + Send + Sync,
    Out: TryFrom<In> + Send + Sync,
    <Out as TryFrom<In>>::Error: Display,
    S: Serde<Out> + Send + Sync,
{
    fn serialize(&self, value: In) -> anyhow::Result<Vec<u8>> {
        self.serde.serialize(
            value
                .try_into()
                .map_err(|err| anyhow!("failed to convert type values: {}", err))?,
        )
    }
}

impl<In, Out, S> Deserializer<In> for Convert<In, Out, S>
where
    In: TryFrom<Out> + Send + Sync,
    Out: TryFrom<In> + Send + Sync,
    <In as TryFrom<Out>>::Error: Display,
    S: Serde<Out> + Send + Sync,
{
    fn deserialize(&self, data: &[u8]) -> anyhow::Result<In> {
        let inn = self.serde.deserialize(data)?;

        inn.try_into()
            .map_err(|err| anyhow!("failed to convert type values: {}", err))
    }
}

/// Implements the [Serializer] and [Deserializer] traits, which use the [serde] crate
/// to serialize and deserialize a message into JSON.
#[cfg(feature = "serde-json")]
#[derive(Debug, Clone, Copy)]
pub struct Json<T>(PhantomData<T>)
where
    T: Serialize + Send + Sync,
    for<'d> T: Deserialize<'d>;

#[cfg(feature = "serde-json")]
impl<T> Default for Json<T>
where
    T: Serialize + Send + Sync,
    for<'d> T: Deserialize<'d>,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "serde-json")]
impl<T> Serializer<T> for Json<T>
where
    T: Serialize + Send + Sync,
    for<'d> T: Deserialize<'d>,
{
    fn serialize(&self, value: T) -> anyhow::Result<Vec<u8>> {
        serde_json::to_vec(&value)
            .map_err(|err| anyhow!("failed to serialize value to json: {}", err))
    }
}

#[cfg(feature = "serde-json")]
impl<T> Deserializer<T> for Json<T>
where
    T: Serialize + Send + Sync,
    for<'d> T: Deserialize<'d>,
{
    fn deserialize(&self, data: &[u8]) -> anyhow::Result<T> {
        serde_json::from_slice(data)
            .map_err(|err| anyhow!("failed to deserialize value from json: {}", err))
    }
}

/// Implements the [Serde] trait  which serializes and deserializes
/// the message using Protobuf format through the [`prost::Message`] trait.
#[cfg(feature = "serde-prost")]
#[derive(Debug, Clone, Copy, Default)]
pub struct Protobuf<T>(PhantomData<T>)
where
    T: prost::Message + Default;

#[cfg(feature = "serde-prost")]
impl<T> Serializer<T> for Protobuf<T>
where
    T: prost::Message + Default,
{
    fn serialize(&self, value: T) -> anyhow::Result<Vec<u8>> {
        Ok(value.encode_to_vec())
    }
}

#[cfg(feature = "serde-prost")]
impl<T> Deserializer<T> for Protobuf<T>
where
    T: prost::Message + Default,
{
    fn deserialize(&self, data: &[u8]) -> anyhow::Result<T> {
        let buf = Bytes::copy_from_slice(data);

        T::decode(buf)
            .map_err(|err| anyhow!("failed to deserialize protobuf message into value: {}", err))
    }
}

/// Implementation of [Serde] traits that uses [ProtoJson](https://protobuf.dev/programming-guides/proto3/#json)
/// as wire protocol.
#[cfg(feature = "serde-prost")]
#[cfg(feature = "serde-json")]
#[derive(Clone, Copy)]
pub struct ProtoJson<T>(PhantomData<T>)
where
    T: prost::Message + Serialize + Default,
    for<'de> T: Deserialize<'de>;

#[cfg(feature = "serde-prost")]
#[cfg(feature = "serde-json")]
impl<T> Default for ProtoJson<T>
where
    T: prost::Message + Serialize + Default,
    for<'de> T: Deserialize<'de>,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[cfg(feature = "serde-prost")]
#[cfg(feature = "serde-json")]
impl<T> Serializer<T> for ProtoJson<T>
where
    T: prost::Message + Serialize + Default,
    for<'de> T: Deserialize<'de>,
{
    fn serialize(&self, value: T) -> anyhow::Result<Vec<u8>> {
        Json::<T>::default().serialize(value)
    }
}

#[cfg(feature = "serde-prost")]
#[cfg(feature = "serde-json")]
impl<T> Deserializer<T> for ProtoJson<T>
where
    T: prost::Message + Serialize + Default,
    for<'de> T: Deserialize<'de>,
{
    fn deserialize(&self, data: &[u8]) -> anyhow::Result<T> {
        Json::<T>::default().deserialize(data)
    }
}
