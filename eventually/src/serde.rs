#[cfg(feature = "serde-json")]
pub mod json;
#[cfg(feature = "serde-prost")]
pub mod prost;

pub trait Serializer<T> {
    fn serialize(&self, value: T) -> Vec<u8>;
}

impl<T, F> Serializer<T> for F
where
    F: Fn(T) -> Vec<u8>,
{
    fn serialize(&self, value: T) -> Vec<u8> {
        self(value)
    }
}

pub trait Deserializer<T> {
    type Error;

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error>;
}

impl<T, Err, F> Deserializer<T> for F
where
    F: Fn(Vec<u8>) -> Result<T, Err>,
{
    type Error = Err;

    fn deserialize(&self, data: Vec<u8>) -> Result<T, Self::Error> {
        self(data)
    }
}

pub trait Serde<T>: Serializer<T> + Deserializer<T> {}

impl<K, T> Serde<T> for K where K: Serializer<T> + Deserializer<T> {}
