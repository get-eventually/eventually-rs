//! This module contains the definition of a [Message] type, which
//! can be used to describe some sort of domain value such as a [Domain Event][crate::event::Envelope],
//! a [Domain Command][crate::command::Envelope], and so on.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a piece of domain data that occurs in the system.
///
/// Each Message has a specific name to it, which should ideally be
/// unique within the domain you're operating in. Example: a Domain Event
/// that represents when an Order was created can have a `name()`: `"OrderWasCreated"`.
pub trait Message {
    /// Returns the domain name of the [Message].
    fn name(&self) -> &'static str;
}

/// Optional metadata to attach to an [Envelope] to provide additional context
/// to the [Message] carried out.
pub type Metadata = HashMap<String, String>;

/// Represents a [Message] packaged for persistance and/or processing by other
/// parts of the system.
///
/// It carries both the actual message (i.e. a payload) and some optional [Metadata].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T>
where
    T: Message,
{
    /// The message payload.
    pub message: T,
    /// Optional metadata to provide additional context to the message.
    pub metadata: Metadata,
}

impl<T> Envelope<T>
where
    T: Message,
{
    /// Adds a new entry in the [Envelope]'s [Metadata].
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
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

        let new_message = message
            .clone()
            .with_metadata("hello_world".into(), "test".into())
            .with_metadata("test_number".into(), 1.to_string());

        println!("Message: {:?}", message);
        println!("New message: {:?}", new_message);

        // Metadata does not affect equality of message.
        assert_eq!(message, new_message);
    }
}
