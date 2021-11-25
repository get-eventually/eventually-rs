#[deny(unsafe_code, unused_qualifications, trivial_casts)]
#[deny(clippy::all)]
#[warn(clippy::pedantic)]
pub mod event;
pub mod metadata;
pub mod test;
pub mod version;

pub type Messages<T> = Vec<Message<T>>;

#[derive(Debug, Clone)]
pub struct Message<T> {
    pub payload: T,
    pub metadata: metadata::Metadata,
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

    pub fn with_metadata<F>(mut self, f: F) -> Self
    where
        F: Fn(metadata::Metadata) -> metadata::Metadata,
    {
        self.metadata = f(self.metadata);
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_with_metadata_does_not_affect_equality() {
        let message = Message {
            payload: "hello",
            metadata: Default::default(),
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
