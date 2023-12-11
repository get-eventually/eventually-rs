//! Contains the [Serde] compatible implementation using Protobuf wire format through [prost].

use std::marker::PhantomData;

use prost::bytes::Bytes;
use prost::Message;

use crate::serde::Serde;

/// Implements the [Serde] trait  which serializes and deserializes
/// the message using Protobuf format through the [Message] trait in [prost].
#[derive(Debug, Clone, Copy, Default)]
pub struct MessageSerde<T>(PhantomData<T>)
where
    T: Message + Default;

impl<T> Serde<T> for MessageSerde<T>
where
    T: Message + Default,
{
    type Error = prost::DecodeError;

    fn serialize(&self, value: T) -> Vec<u8> {
        value.encode_to_vec()
    }

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error> {
        let buf = Bytes::from(data);

        T::decode(buf)
    }
}
