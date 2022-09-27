use std::marker::PhantomData;

use protobuf::{Message, MessageFull};

use crate::serde::{Deserializer, Serializer};

#[derive(Debug, Clone, Copy)]
pub struct ProtobufSerde<T>(PhantomData<T>)
where
    T: Message;

impl<T> Default for ProtobufSerde<T>
where
    T: Message,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Serializer<T> for ProtobufSerde<T>
where
    T: Message,
{
    fn serialize(&self, value: T) -> Vec<u8> {
        value
            .write_to_bytes()
            .expect("serialization from rust type to protobuf format should be successful")
    }
}

impl<T> Deserializer<T> for ProtobufSerde<T>
where
    T: Message,
{
    type Error = protobuf::Error;

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error> {
        T::parse_from_bytes(&data)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProtoJsonSerde<T>(PhantomData<T>)
where
    T: MessageFull;

impl<T> Default for ProtoJsonSerde<T>
where
    T: MessageFull,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Serializer<T> for ProtoJsonSerde<T>
where
    T: MessageFull,
{
    fn serialize(&self, value: T) -> Vec<u8> {
        protobuf_json_mapping::print_to_string(&value)
            .expect("serialization from rust type to protojson should be successful")
            .into_bytes()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProtoJsonDeserializeError {
    #[error("failed to convert input data from bytes to utf-8 string: {0}")]
    Utf8Conversion(#[source] std::str::Utf8Error),
    #[error("failed to parse protobuf message from input string: {0}")]
    ProtobufParse(#[source] protobuf_json_mapping::ParseError),
}

impl<T> Deserializer<T> for ProtoJsonSerde<T>
where
    T: MessageFull,
{
    type Error = ProtoJsonDeserializeError;

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error> {
        let json = std::str::from_utf8(&data).map_err(ProtoJsonDeserializeError::Utf8Conversion)?;

        protobuf_json_mapping::parse_from_str(json)
            .map_err(ProtoJsonDeserializeError::ProtobufParse)
    }
}
