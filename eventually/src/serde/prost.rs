use std::marker::PhantomData;

use prost::bytes::Bytes;
use prost::Message;

use crate::serde::{Deserializer, Serializer};

#[derive(Debug, Clone, Copy, Default)]
pub struct MessageSerde<T>(PhantomData<T>)
where
    T: Message;

impl<T> Serializer<T> for MessageSerde<T>
where
    T: Message,
{
    fn serialize(&self, value: T) -> Vec<u8> {
        value.encode_to_vec()
    }
}

impl<T> Deserializer<T> for MessageSerde<T>
where
    T: Message + Default,
{
    type Error = prost::DecodeError;

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error> {
        let buf = Bytes::from(data);

        T::decode(buf)
    }
}
