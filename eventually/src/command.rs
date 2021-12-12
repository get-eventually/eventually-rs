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

use std::{future::Future, marker::PhantomData, sync::Arc};

use async_trait::async_trait;

use crate::{event, message, message::Message};

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
pub trait Handler<T>: Send + Sync
where
    T: message::Payload + Send + Sync,
{
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
    T: message::Payload + Send + Sync + 'static,
    Err: Send + Sync,
    F: Send + Sync + Fn(Command<T>) -> Fut,
    Fut: Send + Sync + Future<Output = Result<(), Err>>,
{
    type Error = Err;

    async fn handle(&self, command: Command<T>) -> Result<(), Self::Error> {
        self(command).await
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ErrorRecorderError<Cmd, Err> {
    #[error(transparent)]
    CommandFailed(#[from] Cmd),

    #[error("failed to append the error domain event to the event store: {0}")]
    AppendErrorEvent(#[source] Err),
}

#[derive(Clone)]
pub struct ErrorRecorder<T, H, Store, ToStreamIdFn, ToEventFn>
where
    T: message::Payload + Send + Sync,
    H: Handler<T>,
{
    handler: H,
    store: Store,
    to_stream_id: ToStreamIdFn,
    to_error_event: ToEventFn,
    should_capture_when: Option<Arc<dyn Fn(&H::Error) -> bool + Send + Sync>>,
}

impl<T, H, Store, ToStreamIdFn, ToEventFn> ErrorRecorder<T, H, Store, ToStreamIdFn, ToEventFn>
where
    T: message::Payload + Send + Sync,
    H: Handler<T>,
{
    pub fn should_capture_command_error_when<F>(mut self, f: F) -> Self
    where
        F: Fn(&H::Error) -> bool + Send + Sync + 'static,
    {
        self.should_capture_when = Some(Arc::new(f));
        self
    }
}

#[async_trait]
impl<T, H, Store, ToStreamIdFn, ToEventFn> Handler<T>
    for ErrorRecorder<T, H, Store, ToStreamIdFn, ToEventFn>
where
    T: message::Payload + Clone + Send + Sync + 'static,
    H: Handler<T>,
    H::Error: Clone,
    Store: event::Store,
    ToStreamIdFn: Fn(T) -> Store::StreamId + Send + Sync,
    ToEventFn: Fn(T, H::Error) -> Store::Event + Send + Sync,
{
    type Error = ErrorRecorderError<H::Error, Store::AppendError>;

    async fn handle(&self, command: Command<T>) -> Result<(), Self::Error> {
        let result = self.handler.handle(command.clone()).await;

        let error = match result {
            Err(e) => e,
            Ok(()) => return Ok(()), // Early return!
        };

        let should_capture_error = self
            .should_capture_when
            .as_ref()
            .map_or(false, |f| f(&error));

        let stream_id = (self.to_stream_id)(command.payload.clone());
        let error_event = event::Event {
            payload: (self.to_error_event)(command.payload, error.clone()),
            metadata: command.metadata,
        };

        self.store
            .append(
                stream_id,
                event::StreamVersionExpected::Any,
                vec![error_event],
            )
            .await
            .map_err(ErrorRecorderError::AppendErrorEvent)?;

        if should_capture_error {
            return Ok(());
        }

        Err(ErrorRecorderError::CommandFailed(error))
    }
}

pub struct ErrorRecorderBuilderInit<T, H>
where
    T: message::Payload + Send + Sync,
    H: Handler<T>,
{
    handler: H,
    command_marker: PhantomData<T>,
}

impl<T, H> ErrorRecorderBuilderInit<T, H>
where
    T: message::Payload + Send + Sync,
    H: Handler<T>,
{
    #[must_use]
    pub fn using_event_store<Store>(
        self,
        store: Store,
    ) -> ErrorRecorderBuilderWithStore<T, H, Store>
    where
        T: message::Payload + Send + Sync,
        H: Handler<T>,
        Store: event::Store,
    {
        ErrorRecorderBuilderWithStore {
            handler: self.handler,
            command_marker: self.command_marker,
            store,
        }
    }
}

pub struct ErrorRecorderBuilderWithStore<T, H, Store>
where
    T: message::Payload + Send + Sync,
    H: Handler<T>,
    Store: event::Store,
{
    handler: H,
    command_marker: PhantomData<T>,
    store: Store,
}

impl<T, H, Store> ErrorRecorderBuilderWithStore<T, H, Store>
where
    T: message::Payload + Send + Sync,
    H: Handler<T>,
    Store: event::Store,
{
    #[must_use]
    pub fn and_mapping_to<ToStreamIdFn, ToEventFn>(
        self,
        to_stream_id: ToStreamIdFn,
        to_error_event: ToEventFn,
    ) -> ErrorRecorder<T, H, Store, ToStreamIdFn, ToEventFn>
    where
        ToStreamIdFn: Fn(T) -> Store::StreamId + Send + Sync,
        ToEventFn: Fn(T, H::Error) -> Store::Event + Send + Sync,
    {
        ErrorRecorder {
            handler: self.handler,
            store: self.store,
            to_stream_id,
            to_error_event,
            should_capture_when: None,
        }
    }
}

pub trait HandlerExt<T>: Handler<T> + Sized
where
    T: message::Payload + Send + Sync,
{
    fn with_error_recorder(self) -> ErrorRecorderBuilderInit<T, Self> {
        ErrorRecorderBuilderInit {
            handler: self,
            command_marker: PhantomData,
        }
    }
}

impl<T, H> HandlerExt<T> for H
where
    T: message::Payload + Send + Sync,
    H: Handler<T>,
{
}

#[cfg(test)]
mod test_user_domain {
    use async_trait::async_trait;

    use crate::{
        aggregate,
        aggregate::test_user_domain::{User, UserEvent, UserRoot},
        command,
        command::{Command, HandlerExt},
        event,
        event::Event,
        message, test,
        test::store::EventStoreExt,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CreateUser {
        email: String,
        password: String,
    }

    impl message::Payload for CreateUser {
        fn name(&self) -> &'static str {
            "CreateUser"
        }
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

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ChangeUserPassword {
        email: String,
        password: String,
    }

    impl message::Payload for ChangeUserPassword {
        fn name(&self) -> &'static str {
            "ChangeUserPassword"
        }
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

    // Error Recorder tests ----------------------------------------------------------------------------------------- //

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum UserCommandErrorEvent {
        CreateUserFailed {
            command: CreateUser,
            message: String,
        },
        ChangeUserPasswordFailed {
            command: ChangeUserPassword,
            message: String,
        },
    }

    impl message::Payload for UserCommandErrorEvent {
        fn name(&self) -> &'static str {
            match self {
                Self::CreateUserFailed { .. } => "CreateUserFailed",
                Self::ChangeUserPasswordFailed { .. } => "ChangeUserPasswordFailed",
            }
        }
    }

    #[tokio::test]
    async fn it_records_failed_domain_events_successfully() {
        let error_event_store = test::store::InMemory::<String, UserCommandErrorEvent>::default();
        let tracking_event_store = error_event_store.with_recorded_events_tracking();

        test::command_handler::Scenario::when(Command::from(ChangeUserPassword {
            email: "test@test.com".to_owned(),
            password: "new-password".to_owned(),
        }))
        .then_fails()
        .assert_on(|event_store| {
            ChangeUserPasswordHandler(aggregate::EventSourcedRepository::from(event_store))
                .with_error_recorder()
                .using_event_store(tracking_event_store)
                .and_mapping_to(
                    |command| command.email,
                    |command, error| UserCommandErrorEvent::ChangeUserPasswordFailed {
                        command,
                        message: error.to_string(),
                    },
                )
        })
        .await;
    }
}
