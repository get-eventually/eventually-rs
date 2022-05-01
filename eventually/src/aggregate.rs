//! Module containing support for the Aggregate pattern.
//!
//! ## What is an Aggregate?
//!
//! An [Aggregate] is the most important concept in your domain.
//!
//! It represents the entities your business domain is composed of,
//! and the business logic your domain is exposing.
//!
//! For example: in an Order Management bounded-context (e.g. a
//! microservice), the concepts of Order or Customer are two potential
//! [Aggregate]s.
//!
//! Aggregates expose mutations with the concept of **commands**:
//! from the previous example, an Order might expose some commands such as
//! _"Add Order Item"_, or _"Remove Order Item"_, or _"Place Order"_
//! to close the transaction.
//!
//! In Event Sourcing, the Aggregate state is modified by the usage of
//! **Domain Events**, which carry some or all the fields in the state
//! in a certain logical meaning.
//!
//! As such, commands in Event Sourcing will **produce** Domain Events.
//!
//! Aggregates should provide a way to **fold** Domain Events on the
//! current value of the state, to produce the next state.

use crate::message;

mod repository;
mod root;

pub use repository::{EventSourced as EventSourcedRepository, *};
pub use root::*;

/// An Aggregate represents a Domain Model that, through an Aggregate [Root],
/// acts as a _transactional boundary_.
///
/// Aggregates are also used to enforce Domain invariants
/// (i.e. certain constraints or rules that are unique to a specific Domain).
///
/// Since this is an Event-sourced version of the Aggregate pattern,
/// any change to the Aggregate state must be represented through
/// a Domain Event, which is then applied to the current state
/// using the [`Aggregate::apply`] method.
///
/// More on Aggregates can be found here: `<https://www.dddcommunity.org/library/vernon_2011/>`
pub trait Aggregate: Sized + Send + Sync + Clone {
    /// The type used to uniquely identify the Aggregate.
    type Id: ToString + Send + Sync;

    /// The type of Domain Events that interest this Aggregate.
    /// Usually, this type should be an `enum`.
    type Event: message::Message + Send + Sync + Clone;

    /// The error type that can be returned by [`Aggregate::apply`] when
    /// mutating the Aggregate state.
    type Error: Send + Sync;

    /// Returns the unique identifier for the Aggregate instance.
    fn aggregate_id(&self) -> &Self::Id;

    /// Mutates the state of an Aggregate through a Domain Event.
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error>;
}

// The warnings are happening due to usage of the methods only inside #[cfg(test)]
#[allow(dead_code)]
#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_user_domain {
    use std::borrow::{Borrow, BorrowMut};

    use crate::{aggregate, aggregate::Root, event, message};

    #[derive(Debug, Clone)]
    pub(crate) struct User {
        email: String,
        password: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) enum UserEvent {
        WasCreated { email: String, password: String },
        PasswordWasChanged { password: String },
    }

    impl message::Message for UserEvent {
        fn name(&self) -> &'static str {
            match self {
                UserEvent::WasCreated { .. } => "UserWasCreated",
                UserEvent::PasswordWasChanged { .. } => "UserPasswordWasChanged",
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum UserError {
        #[error("provided email was empty")]
        EmptyEmail,

        #[error("provided password was empty")]
        EmptyPassword,

        #[error("user was not yet created")]
        NotYetCreated,

        #[error("user was already created")]
        AlreadyCreated,
    }

    impl aggregate::Aggregate for User {
        type Id = String;
        type Event = UserEvent;
        type Error = UserError;

        fn aggregate_id(&self) -> &Self::Id {
            &self.email
        }

        fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
            match state {
                None => match event {
                    UserEvent::WasCreated { email, password } => Ok(User { email, password }),
                    UserEvent::PasswordWasChanged { .. } => Err(UserError::NotYetCreated),
                },
                Some(mut state) => match event {
                    UserEvent::PasswordWasChanged { password } => {
                        state.password = password;
                        Ok(state)
                    }
                    UserEvent::WasCreated { .. } => Err(UserError::AlreadyCreated),
                },
            }
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct UserRoot(aggregate::Context<User>);

    impl From<aggregate::Context<User>> for UserRoot {
        fn from(ctx: aggregate::Context<User>) -> Self {
            Self(ctx)
        }
    }

    impl Borrow<aggregate::Context<User>> for UserRoot {
        fn borrow(&self) -> &aggregate::Context<User> {
            &self.0
        }
    }

    impl BorrowMut<aggregate::Context<User>> for UserRoot {
        fn borrow_mut(&mut self) -> &mut aggregate::Context<User> {
            &mut self.0
        }
    }

    impl Root<User> for UserRoot {
        fn type_name() -> &'static str {
            "User"
        }
    }

    impl UserRoot {
        pub(crate) fn create(email: String, password: String) -> Result<Self, UserError> {
            if email.is_empty() {
                return Err(UserError::EmptyEmail);
            }

            if password.is_empty() {
                return Err(UserError::EmptyPassword);
            }

            Ok(UserRoot::record_new(event::Envelope::from(
                UserEvent::WasCreated { email, password },
            ))?)
        }

        pub(crate) fn change_password(&mut self, password: String) -> Result<(), UserError> {
            if password.is_empty() {
                return Err(UserError::EmptyPassword);
            }

            self.record_that(event::Envelope::from(UserEvent::PasswordWasChanged {
                password,
            }))?;

            Ok(())
        }
    }
}

#[allow(clippy::semicolon_if_nothing_returned)] // False positives :shrugs:
#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::{
        aggregate,
        aggregate::test_user_domain::{User, UserEvent, UserRoot},
        aggregate::Repository,
        event, test,
        test::store::EventStoreExt,
        version,
    };

    #[tokio::test]
    async fn repository_persists_new_aggregate_root() {
        let event_store = test::store::InMemory::<String, UserEvent>::default();
        let tracking_event_store = event_store.with_recorded_events_tracking();
        let user_repository = aggregate::EventSourcedRepository::<User, UserRoot, _>::from(
            tracking_event_store.clone(),
        );

        let email = "test@email.com".to_owned();
        let password = "not-a-secret".to_owned();

        let mut user = UserRoot::create(email.clone(), password.clone())
            .expect("user should be created successfully");

        user_repository
            .store(&mut user)
            .await
            .expect("user should be stored successfully");

        let expected_events = vec![event::Persisted {
            stream_id: email.clone(),
            version: 1,
            event: event::Envelope::from(UserEvent::WasCreated { email, password }),
        }];

        assert_eq!(expected_events, tracking_event_store.recorded_events());
    }

    #[tokio::test]
    async fn repository_retrieves_the_aggregate_root_and_stores_new_events() {
        let event_store = test::store::InMemory::<String, UserEvent>::default();
        let tracking_event_store = event_store.with_recorded_events_tracking();
        let user_repository = aggregate::EventSourcedRepository::<User, UserRoot, _>::from(
            tracking_event_store.clone(),
        );

        let email = "test@email.com".to_owned();
        let password = "not-a-secret".to_owned();

        let mut user = UserRoot::create(email.clone(), password.clone())
            .expect("user should be created successfully");

        user_repository
            .store(&mut user)
            .await
            .expect("user should be stored successfully");

        // Reset the event recorded while storing the User for the first time.
        tracking_event_store.reset_recorded_events();

        let mut user = user_repository
            .get(&email)
            .await
            .expect("user should be retrieved from the repository");

        let new_password = "new-password".to_owned();

        user.change_password(new_password.clone())
            .expect("user password should be changed successfully");

        user_repository
            .store(&mut user)
            .await
            .expect("new user version should be stored successfully");

        let expected_events = vec![event::Persisted {
            stream_id: email.clone(),
            version: 2,
            event: event::Envelope::from(UserEvent::PasswordWasChanged {
                password: new_password,
            }),
        }];

        assert_eq!(expected_events, tracking_event_store.recorded_events());
    }

    #[tokio::test]
    async fn repository_returns_conflict_error_from_store_when_data_race_happens() {
        let event_store = test::store::InMemory::<String, UserEvent>::default();
        let user_repository =
            aggregate::EventSourcedRepository::<User, UserRoot, _>::from(event_store.clone());

        let email = "test@email.com".to_owned();
        let password = "not-a-secret".to_owned();

        let mut user = UserRoot::create(email.clone(), password.clone())
            .expect("user should be created successfully");

        // We need to clone the UserRoot instance to get the list
        // of uncommitted events from the Context twice.
        let mut cloned_user = user.clone();

        // Saving the first User to the Repository.
        user_repository
            .store(&mut user)
            .await
            .expect("user should be stored successfully");

        // Simulating data race by duplicating the call to the Repository
        // with the same UserRoot instance that has already been committeed.
        let error = user_repository
            .store(&mut cloned_user)
            .await
            .expect_err("the repository should fail on the second store call with the cloned user");

        let error: Box<dyn Error> = error.into();

        // Have no idea how to fix this one...
        #[allow(clippy::redundant_closure_for_method_calls)]
        {
            assert!(error
                .source()
                .map_or(false, |src| src.is::<version::ConflictError>()));
        }
    }
}
