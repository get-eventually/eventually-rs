#![allow(unused)]

use eventually::aggregate::OptionalAggregate;

#[derive(Debug, Clone, PartialEq)]
/// Person is the main entity in our small domain.
struct Person {
    name: String,
    last_name: String,
    version: u64,
    married_to: Option<String>,
}

#[derive(Debug)]
/// Event contains all the possible domain events related to Person.
enum Event {
    WasBorn { name: String, last_name: String },
    ChangedName { name: String, last_name: String },
    GotMarriedTo(String),
}

#[derive(Debug, PartialEq, Eq)]
/// Error contains all the possible errors that can be raised while
/// applying domain events to a Person state.
enum Error {
    PersonNotFound,
    PersonAlreadyExists,
}

impl OptionalAggregate for Person {
    type State = Self;
    type Event = Event;
    type Error = Error;

    fn initial(event: Self::Event) -> Result<Self::State, Self::Error> {
        match event {
            Event::WasBorn { name, last_name } => Ok(Person {
                name,
                last_name,
                married_to: None,
                version: 1,
            }),
            _ => Err(Error::PersonNotFound),
        }
    }

    fn apply_next(mut state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        state.version += 1;

        match event {
            Event::WasBorn { .. } => return Err(Error::PersonAlreadyExists),
            Event::ChangedName { name, last_name } => {
                state.name = name;
                state.last_name = last_name;
            }
            Event::GotMarriedTo(person) => {
                state.married_to = Some(person);
            }
        };

        Ok(state)
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    use eventually::{aggregate::optional::AsAggregate, AggregateExt};

    #[test]
    fn it_folds_multiple_events_correctly() {
        let result = AsAggregate::<Person>::fold(
            None,
            vec![
                Event::WasBorn {
                    name: "John".to_string(),
                    last_name: "Doe".to_string(),
                },
                Event::ChangedName {
                    name: "John".to_string(),
                    last_name: "Carpenter".to_string(),
                },
                Event::GotMarriedTo("Somebody".to_string()),
            ]
            .into_iter(),
        );

        assert_eq!(
            result,
            Ok(Some(Person {
                version: 3,
                name: "John".to_string(),
                last_name: "Carpenter".to_string(),
                married_to: Some("Somebody".to_string()),
            }))
        );
    }
}
