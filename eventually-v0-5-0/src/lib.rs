#[deny(unsafe_code, unused_qualifications, trivial_casts)]
#[deny(clippy::all)]
#[warn(clippy::pedantic)]
pub mod event;
pub mod test;
pub mod version;

use std::collections::HashMap;

pub type Messages<T> = Vec<Message<T>>;

#[derive(Debug, Clone)]
pub struct Message<T> {
    pub payload: T,
    pub metadata: HashMap<String, String>,
}

impl<T> Message<T> {
    pub fn map_into<U>(self) -> Message<U>
    where
        T: Into<U>,
    {
        Message {
            payload: self.payload.into(),
            metadata: self.metadata,
        }
    }
}

impl<T> From<T> for Message<T> {
    fn from(payload: T) -> Self {
        Message {
            payload,
            metadata: Default::default(),
        }
    }
}

impl<T> PartialEq for Message<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Message<T>) -> bool {
        self.payload == other.payload
    }
}
