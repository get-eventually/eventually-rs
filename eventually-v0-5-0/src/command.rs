//! Module containing support for Domain [Command]s.
//!
//! Following the Domain-driven Design definition, a [Command] expresses the
//! intent of an Actor (e.g. a Customer, a User, a System, etc.) to modify
//! the state of the system in some way.
//!
//! To modify the state of the system through a [Command], you must
//! implement a Command [Handler] which, in an Event-sourced system,
//! should make use of an [Aggregate] to evaluate the validity of the Command
//! submitted, and emit Domain [Event]s as a result (through the Event [Store]).
//!
//! Check out the type documentation exported in this module.

use std::future::Future;

use async_trait::async_trait;

use crate::Message;

/// A Command represents an intent by an Actor (e.g. a User, or a System)
/// to mutate the state of the system.
///
/// In an event-sourced system, a Command is represented as a [Message].
pub type Command<T> = Message<T>;

/// A software component that is able to handle [Command]s of a certain type,
/// and mutate the state as a result of the command handling, or fail.
///
/// In an event-sourced system, the [Command] Handler
/// should use an [Aggregate][crate::aggregate::Aggregate] to evaluate
/// a [Command] to ensure business invariants are respected.
#[async_trait]
pub trait Handler<T>: Send + Sync {
    /// The error type returned by the Handler while handling a [Command].
    type Error: Send + Sync;

    /// Handles a [Command] and returns an error if the handling has failed.
    ///
    /// Since [Command]s are solely modifying the state of the system,
    /// they do not return anything to the caller but the result of the operation
    /// (expressed by a [Result] type).
    async fn handle(&self, command: Command<T>) -> Result<(), Self::Error>;
}

#[async_trait]
impl<T, Err, F, Fut> Handler<T> for F
where
    T: Send + Sync + 'static,
    Err: Send + Sync,
    F: Send + Sync + Fn(Command<T>) -> Fut,
    Fut: Send + Sync + Future<Output = Result<(), Err>>,
{
    type Error = Err;

    async fn handle(&self, command: Command<T>) -> Result<(), Self::Error> {
        self(command).await
    }
}

#[cfg(test)]
mod test_user_domain {
    use async_trait::async_trait;

    use crate::{
        aggregate,
        aggregate::test_user_domain::{User, UserEvent, UserRoot},
        command,
        command::Command,
        event,
        event::Event,
        test,
    };

    struct CreateUser {
        email: String,
        password: String,
    }

    struct CreateUserHandler<R>(R)
    where
        R: aggregate::Repository<User, UserRoot>;

    #[async_trait]
    impl<R> command::Handler<CreateUser> for CreateUserHandler<R>
    where
        R: aggregate::Repository<User, UserRoot>,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        type Error = anyhow::Error;

        async fn handle(&self, command: Command<CreateUser>) -> Result<(), Self::Error> {
            let command = command.payload;
            let mut user = UserRoot::create(command.email, command.password)?;

            self.0.store(&mut user).await?;

            Ok(())
        }
    }

    struct ChangeUserPassword {
        email: String,
        password: String,
    }

    struct ChangeUserPasswordHandler<R>(R)
    where
        R: aggregate::Repository<User, UserRoot>;

    #[async_trait]
    impl<R> command::Handler<ChangeUserPassword> for ChangeUserPasswordHandler<R>
    where
        R: aggregate::Repository<User, UserRoot>,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        type Error = anyhow::Error;

        async fn handle(&self, command: Command<ChangeUserPassword>) -> Result<(), Self::Error> {
            let command = command.payload;

            let mut user = self.0.get(&command.email).await?;

            user.change_password(command.password)?;

            self.0.store(&mut user).await?;

            Ok(())
        }
    }

    #[tokio::test]
    async fn it_creates_a_new_user_successfully() {
        test::command_handler::Scenario::when(Command::from(CreateUser {
            email: "test@test.com".to_owned(),
            password: "not-a-secret".to_owned(),
        }))
        .then(vec![event::Persisted {
            stream_id: "test@test.com".to_owned(),
            version: 1,
            payload: Event::from(UserEvent::WasCreated {
                email: "test@test.com".to_owned(),
                password: "not-a-secret".to_owned(),
            }),
        }])
        .assert_on(|event_store| {
            CreateUserHandler(aggregate::EventSourcedRepository::from(event_store))
        })
        .await;
    }

    #[tokio::test]
    async fn it_fails_to_create_an_user_if_it_still_exists() {
        test::command_handler::Scenario::given(vec![event::Persisted {
            stream_id: "test@test.com".to_owned(),
            version: 1,
            payload: Event::from(UserEvent::WasCreated {
                email: "test@test.com".to_owned(),
                password: "not-a-secret".to_owned(),
            }),
        }])
        .when(Command::from(CreateUser {
            email: "test@test.com".to_owned(),
            password: "not-a-secret".to_owned(),
        }))
        .then_fails()
        .assert_on(|event_store| {
            CreateUserHandler(aggregate::EventSourcedRepository::from(event_store))
        })
        .await;
    }

    #[tokio::test]
    async fn it_updates_the_password_of_an_existing_user() {
        test::command_handler::Scenario::given(vec![event::Persisted {
            stream_id: "test@test.com".to_owned(),
            version: 1,
            payload: Event::from(UserEvent::WasCreated {
                email: "test@test.com".to_owned(),
                password: "not-a-secret".to_owned(),
            }),
        }])
        .when(Command::from(ChangeUserPassword {
            email: "test@test.com".to_owned(),
            password: "new-password".to_owned(),
        }))
        .then(vec![event::Persisted {
            stream_id: "test@test.com".to_owned(),
            version: 2,
            payload: Event::from(UserEvent::PasswordWasChanged {
                password: "new-password".to_owned(),
            }),
        }])
        .assert_on(|event_store| {
            ChangeUserPasswordHandler(aggregate::EventSourcedRepository::from(event_store))
        })
        .await;
    }

    #[tokio::test]
    async fn it_fails_to_update_the_password_if_the_user_does_not_exist() {
        test::command_handler::Scenario::when(Command::from(ChangeUserPassword {
            email: "test@test.com".to_owned(),
            password: "new-password".to_owned(),
        }))
        .then_fails()
        .assert_on(|event_store| {
            ChangeUserPasswordHandler(aggregate::EventSourcedRepository::from(event_store))
        })
        .await;
    }
}
