use serde::{Deserialize, Serialize};

use crate::metadata::Metadata;

pub trait Payload {
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T>
where
    T: Payload,
{
    pub payload: T,
    pub metadata: Metadata,
}

impl<T> Message<T>
where
    T: Payload,
{
    pub fn with_metadata<F>(mut self, f: F) -> Self
    where
        F: Fn(Metadata) -> Metadata,
    {
        self.metadata = f(self.metadata);
        self
    }
}

impl<T> From<T> for Message<T>
where
    T: Payload,
{
    fn from(payload: T) -> Self {
        Message {
            payload,
            metadata: Metadata::default(),
        }
    }
}

impl<T> PartialEq for Message<T>
where
    T: Payload + PartialEq,
{
    fn eq(&self, other: &Message<T>) -> bool {
        self.payload == other.payload
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct StringPayload(pub(crate) &'static str);

    impl Payload for StringPayload {
        fn name(&self) -> &'static str {
            "string_payload"
        }
    }

    #[test]
    fn message_with_metadata_does_not_affect_equality() {
        let message = Message {
            payload: StringPayload("hello"),
            metadata: Metadata::default(),
        };

        let new_message = message.clone().with_metadata(|metadata| {
            metadata
                .add_string("hello_world".to_owned(), "test".to_owned())
                .add_integer("test_number".to_owned(), 1)
        });

        println!("Message: {:?}", message);
        println!("New message: {:?}", new_message);

        // Metadata does not affect equality of message.
        assert_eq!(message, new_message);
    }
}
