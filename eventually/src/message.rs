use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub trait Message {
    fn name(&self) -> &'static str;
}

pub type Metadata = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T>
where
    T: Message,
{
    pub message: T,
    pub metadata: Metadata,
}

impl<T> Envelope<T>
where
    T: Message,
{
    #[must_use]
    pub fn and_metadata<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Metadata),
    {
        f(&mut self.metadata);
        self
    }
}

impl<T> From<T> for Envelope<T>
where
    T: Message,
{
    fn from(message: T) -> Self {
        Envelope {
            message,
            metadata: Metadata::default(),
        }
    }
}

impl<T> PartialEq for Envelope<T>
where
    T: Message + PartialEq,
{
    fn eq(&self, other: &Envelope<T>) -> bool {
        self.message == other.message
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct StringMessage(pub(crate) &'static str);

    impl Message for StringMessage {
        fn name(&self) -> &'static str {
            "string_payload"
        }
    }

    #[test]
    fn message_with_metadata_does_not_affect_equality() {
        let message = Envelope {
            message: StringMessage("hello"),
            metadata: Metadata::default(),
        };

        let new_message = message.clone().and_metadata(|m| {
            m.insert("hello_world".to_owned(), "test".to_owned());
            m.insert("test_number".to_owned(), 1.to_string());
        });

        println!("Message: {:?}", message);
        println!("New message: {:?}", new_message);

        // Metadata does not affect equality of message.
        assert_eq!(message, new_message);
    }
}
