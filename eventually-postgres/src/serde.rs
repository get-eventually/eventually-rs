use std::marker::PhantomData;

pub type ByteArray = Vec<u8>;

pub trait Serializer<T> {
    fn serialize(&self, value: T) -> ByteArray;
}

impl<T, F> Serializer<T> for F
where
    F: Fn(T) -> ByteArray,
{
    fn serialize(&self, value: T) -> ByteArray {
        self(value)
    }
}

pub trait Deserializer<T> {
    type Error;

    fn deserialize(&self, value: ByteArray) -> Result<T, Self::Error>;
}

impl<T, Err, F> Deserializer<T> for F
where
    F: Fn(ByteArray) -> Result<T, Err>,
{
    type Error = Err;

    fn deserialize(&self, value: ByteArray) -> Result<T, Self::Error> {
        self(value)
    }
}

pub trait Serde<T>: Serializer<T> + Deserializer<T> {}

impl<K, T> Serde<T> for K where K: Serializer<T> + Deserializer<T> {}
