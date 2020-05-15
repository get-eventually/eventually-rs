// #![allow(unused)]

// use eventually::aggregate::Aggregate;

// #[derive(Debug, Clone, PartialEq)]
// /// Person is the main entity in our small domain.
// struct Person {
//     name: String,
//     last_name: String,
//     version: u64,
//     married_to: Option<String>,
// }

// #[derive(Debug)]
// /// Event contains all the possible domain events related to Person.
// enum Event {
//     WasBorn { name: String, last_name: String },
//     ChangedName { name: String, last_name: String },
//     GotMarriedTo(String),
// }

// #[derive(Debug, PartialEq, Eq)]
// /// Error contains all the possible errors that can be raised while
// /// applying domain events to a Person state.
// enum Error {
//     PersonNotFound,
//     PersonAlreadyExists,
// }

// impl Aggregate for Person {
//     type State = Option<Self>;
//     type Event = Event;
//     type Error = Error;

//     fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
//         Ok(Some(match state {
//             Some(state) => state.apply_next(event)?,
//             None => Self::apply_first(event)?,
//         }))
//     }
// }

// impl Person {
//     fn apply_first(event: Event) -> Result<Self, Error> {
//         match event {
//             Event::WasBorn { name, last_name } => Ok(Person {
//                 name,
//                 last_name,
//                 married_to: None,
//                 version: 1,
//             }),
//             _ => Err(Error::PersonNotFound),
//         }
//     }

//     fn apply_next(mut self, event: Event) -> Result<Self, Error> {
//         self.version += 1;

//         match event {
//             Event::WasBorn { .. } => return Err(Error::PersonAlreadyExists),
//             Event::ChangedName { name, last_name } => {
//                 self.name = name;
//                 self.last_name = last_name;
//             }
//             Event::GotMarriedTo(person) => {
//                 self.married_to = Some(person);
//             }
//         };

//         Ok(self)
//     }
// }

// fn main() {}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use eventually::AggregateExt;

//     #[test]
//     fn it_fails_when_applying_non_apply_first_events_to_empty_state() {
//         assert_eq!(
//             Person::apply(
//                 None,
//                 Event::ChangedName {
//                     name: "John".to_string(),
//                     last_name: "Carpenter".to_string(),
//                 }
//             ),
//             Err(Error::PersonNotFound)
//         );
//     }

//     #[test]
//     fn it_applies_event_successfully() {
//         let mut result = Person::apply(
//             None,
//             Event::WasBorn {
//                 name: "John".to_string(),
//                 last_name: "Doe".to_string(),
//             },
//         );

//         assert_eq!(
//             result,
//             Ok(Some(Person {
//                 version: 1,
//                 name: "John".to_string(),
//                 last_name: "Doe".to_string(),
//                 married_to: None,
//             }))
//         );

//         result = result.and_then(|state| {
//             Person::apply(
//                 state,
//                 Event::ChangedName {
//                     name: "John".to_string(),
//                     last_name: "Carpenter".to_string(),
//                 },
//             )
//         });

//         assert_eq!(
//             result,
//             Ok(Some(Person {
//                 version: 2,
//                 name: "John".to_string(),
//                 last_name: "Carpenter".to_string(),
//                 married_to: None,
//             }))
//         );

//         result = result
//             .and_then(|state| Person::apply(state, Event::GotMarriedTo("Somebody".to_string())));

//         assert_eq!(
//             result,
//             Ok(Some(Person {
//                 version: 3,
//                 name: "John".to_string(),
//                 last_name: "Carpenter".to_string(),
//                 married_to: Some("Somebody".to_string()),
//             }))
//         );
//     }

//     #[test]
//     fn it_folds_multiple_events_correctly() {
//         let result = Person::fold(
//             None,
//             vec![
//                 Event::WasBorn {
//                     name: "John".to_string(),
//                     last_name: "Doe".to_string(),
//                 },
//                 Event::ChangedName {
//                     name: "John".to_string(),
//                     last_name: "Carpenter".to_string(),
//                 },
//                 Event::GotMarriedTo("Somebody".to_string()),
//             ]
//             .into_iter(),
//         );

//         assert_eq!(
//             result,
//             Ok(Some(Person {
//                 version: 3,
//                 name: "John".to_string(),
//                 last_name: "Carpenter".to_string(),
//                 married_to: Some("Somebody".to_string()),
//             }))
//         );
//     }

//     #[test]
//     fn it_fails_when_one_of_the_events_cannot_be_applied() {
//         let result = Person::fold(
//             None,
//             vec![
//                 Event::WasBorn {
//                     name: "John".to_string(),
//                     last_name: "Doe".to_string(),
//                 },
//                 // Born twice, should return an Error::PersonAlreadyExists
//                 Event::WasBorn {
//                     name: "John".to_string(),
//                     last_name: "Doe".to_string(),
//                 },
//                 Event::ChangedName {
//                     name: "John".to_string(),
//                     last_name: "Carpenter".to_string(),
//                 },
//             ]
//             .into_iter(),
//         );

//         assert_eq!(result, Err(Error::PersonAlreadyExists));
//     }
// }
