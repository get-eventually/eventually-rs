#[cfg(feature = "serde-json")]
pub mod json;
#[cfg(feature = "serde-prost")]
pub mod prost;

/// A [Serde] can be used to serialize into and deserialize from a given type
/// into a wire format, such as [JSON][json] or [Protobuf][prost].
pub trait Serde<T>: Send + Sync {
    /// The error returned by the [Serde::deserialize] method.
    type Error: Send + Sync;

    /// Serializes the given value into the wire format supported by this [Serde].
    fn serialize(&self, value: T) -> Vec<u8>;

    /// Deserializes the given value from a message encoded in the wire format
    /// supported by this [Serde].
    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error>;
}
