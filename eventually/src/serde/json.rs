use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::serde::{Deserializer, Serializer};

#[derive(Debug, Clone, Copy)]
pub struct Json<T>(PhantomData<T>);

impl<T> Default for Json<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Serializer<T> for Json<T>
where
    T: Serialize,
{
    fn serialize(&self, value: T) -> Vec<u8> {
        serde_json::to_vec(&value).expect("json serialization should not fail")
    }
}

impl<T> Deserializer<T> for Json<T>
where
    for<'d> T: Deserialize<'d>,
{
    type Error = serde_json::Error;

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error> {
        serde_json::from_slice(&data)
    }
}
