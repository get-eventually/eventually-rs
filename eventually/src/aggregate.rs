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

use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use futures::TryStreamExt;

use crate::{event, event::Event, version::Version};

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
pub trait Aggregate: Sized + Send + Sync {
    /// The type used to uniquely identify the Aggregate.
    type Id: Send + Sync;

    /// The type of Domain Events that interest this Aggregate.
    /// Usually, this type should be an `enum`.
    type Event: Send + Sync;

    /// The error type that can be returned by [`Aggregate::apply`] when
    /// mutating the Aggregate state.
    type Error: Send + Sync;

    /// Returns the unique identifier for the Aggregate instance.
    fn id(&self) -> &Self::Id;

    /// Mutates the state of an Aggregate through a Domain Event.
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error>;
}

/// A context object that should be used by the Aggregate [Root] methods to
/// access the [Aggregate] state and to record new Domain [Event]s.
#[derive(Debug, Clone)]
#[must_use]
pub struct Context<T>
where
    T: Aggregate,
{
    aggregate: T,
    version: Version,
    recorded_events: Vec<Event<T::Event>>,
}

impl<T> Context<T>
where
    T: Aggregate,
{
    /// Returns read access to the [Aggregate] state.
    pub fn aggregate(&self) -> &T {
        &self.aggregate
    }

    /// Returns the list of uncommitted, recorded Domain [Event]s from the [Context]
    /// and resets the internal list to its default value.
    fn take_uncommitted_events(&mut self) -> Vec<Event<T::Event>> {
        std::mem::take(&mut self.recorded_events)
    }

    /// Creates a new [Context] instance from a Domain [Event]
    /// while rehydrating an [Aggregate].
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    fn rehydrate_from(event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.payload)?,
            recorded_events: Vec::default(),
        })
    }

    /// Applies a new Domain [Event] to the [Context] while rehydrating
    /// an [Aggregate].
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    fn apply_rehydrated_event(mut self, event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        self.aggregate = T::apply(Some(self.aggregate), event.payload)?;
        self.version += 1;

        Ok(self)
    }
}

impl<T> Context<T>
where
    T: Aggregate + Clone,
    T::Event: Clone,
{
    /// Creates a new [Aggregate] instance by applying the specified
    /// Domain [Event], and returns a [Context] reference with the Aggregate
    /// instance in it.
    ///
    /// This method should be used inside Aggregate [Root] methods
    /// to create new [Root] instances:
    /// ```text
    /// use eventually::{
    ///     event::Event,
    ///     aggregate::Root,
    ///     aggregate,
    /// };
    ///
    /// let my_aggregate_root = MyAggregateRoot::from(
    ///     aggregate::Context::record_new(
    ///         Event::from(MyDomainEvent { /* something */ })
    ///     )?
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    pub fn record_new(event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.payload.clone())?,
            recorded_events: vec![event],
        })
    }

    /// Records a change to the [Aggregate], expressed by the specified
    /// Domain [Event].
    ///
    /// This method should be used inside Aggregate [Root] methods
    /// to update the [Aggregate] state:
    /// ```text
    /// use eventually::{
    ///     event::Event,
    ///     aggregate::Root,
    /// };
    ///
    /// impl MyAggregateRoot {
    ///     pub fn update_name(&mut self, name: String) -> Result<(), MyAggregateError> {
    ///         if name.is_empty() {
    ///             return Err(MyAggregateError::NameIsEmpty);
    ///         }
    ///
    ///         self.ctx_mut().record_that(
    ///             Event::from(MyAggergateEvent::NameWasChanged { name })
    ///         )
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    pub fn record_that(&mut self, event: Event<T::Event>) -> Result<(), T::Error> {
        self.aggregate = T::apply(Some(self.aggregate.clone()), event.payload.clone())?;
        self.recorded_events.push(event);
        self.version += 1;

        Ok(())
    }
}

/// An Aggregate Root represents the Domain Entity object used to
/// load and save an [Aggregate] from and to a [Repository], and
/// to perform actions that may result in new Domain [Event]s
/// to change the state of the Aggregate.
///
/// An Aggregate Root implementation should only depend on [Context],
/// and implement the `From<Context<AggregateType>>` trait. The Aggregate state
/// and list of Domain Events recorded are handled by the Context object itself.
///
/// ```text
/// #[derive(Debug, Clone)]
/// struct MyAggregateRoot(Context<MyAggregate>);
///
/// impl From<Context<MyAggregate>> for MyAggregateRoot {
///     fn from(ctx: Context<MyAggregate>) -> Self {
///         Self(ctx)
///     }
/// }
///
/// // Implement the Aggregate Root interface by providing
/// // read/write access to the Context object.
/// impl aggregate::Root<MyAggregate> for MyAggregateRoot {
///     fn ctx(&self) -> &Context<MyAggregate> {
///         &self.0
///     }
///
///     fn ctx_mut(&mut self) -> &mut Context<MyAggregate> {
///         &mut self.0
///     }
/// }
/// ```
///
/// For more information on how to record Domain Events using an Aggregate Root,
/// please check [`Context::record_that`] method documentation.
pub trait Root<T>: From<Context<T>> + Send + Sync
where
    T: Aggregate,
{
    /// Provides read access to an [Aggregate] [Root] [Context].
    fn ctx(&self) -> &Context<T>;

    /// Provides write access to an [Aggregate] [Root] [Context].
    fn ctx_mut(&mut self) -> &mut Context<T>;

    /// Convenience method to resolve the [Aggregate] unique identifier
    /// from the Aggregate Root instance.
    fn aggregate_id<'a>(&'a self) -> &'a T::Id
    where
        T: 'a,
    {
        self.ctx().aggregate().id()
    }
}

/// A Repository is an object that allows to load and save
/// an [Aggregate Root][Root] from and to a persistent data store.
#[async_trait]
pub trait Repository<T, R>: Send + Sync
where
    T: Aggregate,
    R: Root<T>,
{
    /// The error type that can be returned by the Repository implementation
    /// during loading or storing of an Aggregate Root.
    type Error;

    /// Loads an Aggregate Root instance from the data store,
    /// referenced by its unique identifier.
    async fn get(&self, id: &T::Id) -> Result<R, Self::Error>;

    /// Stores a new version of an Aggregate Root instance to the data store.
    async fn store(&self, root: &mut R) -> Result<(), Self::Error>;
}

/// List of possible errors that can be returned by an [`EventSourcedRepository`] method.
#[derive(Debug, thiserror::Error)]
pub enum EventSourcedRepositoryError<E, SE, AE> {
    /// This error is retured by [`EventSourcedRepository::get`] when the
    /// desired Aggregate [Root] could not be found in the data store.
    #[error("aggregate root was not found")]
    AggregateRootNotFound,

    /// This error is returned by [`EventSourcedRepository::get`] when
    /// the desired [Aggregate] returns an error while applying a Domain Event
    /// from the Event [Store][`event::Store`] during the _rehydration_ phase.
    ///
    /// This usually implies the Event Stream for the Aggregate
    /// contains corrupted or unexpected data.
    #[error("failed to rehydrate aggregate from event stream: {0}")]
    RehydrateAggregate(#[source] E),

    /// This error is returned by [`EventSourcedRepository::get`] when the
    /// [Event Store][`event::Store`] used by the Repository returns
    /// an unexpected error while streaming back the Aggregate's Event Stream.
    #[error("event store failed while streaming events: {0}")]
    StreamFromStore(#[source] SE),

    /// This error is returned by [`EventSourcedRepository::store`] when
    /// the [Event Store][`event::Store`] used by the Repository returns
    /// an error while saving the uncommitted Domain Events
    /// to the Aggregate's Event Stream.
    #[error("event store failed while appending events: {0}")]
    AppendToStore(#[source] AE),
}

/// An Event-sourced implementation of the [Repository] interface.
///
/// It uses an [Event Store][`event::Store`] instance to stream Domain Events
/// for a particular Aggregate, and append uncommitted Domain Events
/// recorded by an Aggregate Root.
pub struct EventSourcedRepository<T, R, S>
where
    T: Aggregate,
    R: Root<T>,
    S: event::Store<StreamId = T::Id, Event = T::Event>,
{
    store: S,
    aggregate: PhantomData<T>,
    aggregate_root: PhantomData<R>,
}

impl<T, R, S> From<S> for EventSourcedRepository<T, R, S>
where
    T: Aggregate,
    R: Root<T>,
    S: event::Store<StreamId = T::Id, Event = T::Event>,
{
    fn from(store: S) -> Self {
        Self {
            store,
            aggregate: PhantomData,
            aggregate_root: PhantomData,
        }
    }
}

#[async_trait]
impl<T, R, S> Repository<T, R> for EventSourcedRepository<T, R, S>
where
    T: Aggregate,
    T::Id: Clone,
    T::Error: Debug,
    R: Root<T>,
    S: event::Store<StreamId = T::Id, Event = T::Event>,
{
    type Error = EventSourcedRepositoryError<T::Error, S::StreamError, S::AppendError>;

    async fn get(&self, id: &T::Id) -> Result<R, Self::Error> {
        let ctx = self
            .store
            .stream(id, event::VersionSelect::All)
            .map_ok(|event| event.payload)
            .map_err(EventSourcedRepositoryError::StreamFromStore)
            .try_fold(None, |ctx: Option<Context<T>>, event| async {
                let new_ctx_result = match ctx {
                    None => Context::rehydrate_from(event),
                    Some(ctx) => ctx.apply_rehydrated_event(event),
                };

                let new_ctx =
                    new_ctx_result.map_err(EventSourcedRepositoryError::RehydrateAggregate)?;

                Ok(Some(new_ctx))
            })
            .await?;

        ctx.ok_or(EventSourcedRepositoryError::AggregateRootNotFound)
            .map(R::from)
    }

    async fn store(&self, root: &mut R) -> Result<(), Self::Error> {
        let events_to_commit = root.ctx_mut().take_uncommitted_events();
        let aggregate_id = root.aggregate_id();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let current_event_stream_version = root.ctx().version - (events_to_commit.len() as Version);

        let new_version = self
            .store
            .append(
                aggregate_id.clone(),
                event::StreamVersionExpected::MustBe(current_event_stream_version),
                events_to_commit,
            )
            .await
            .map_err(EventSourcedRepositoryError::AppendToStore)?;

        root.ctx_mut().version = new_version;

        Ok(())
    }
}

// The warnings are happening due to usage of the methods only inside #[cfg(test)]
#[allow(dead_code)]
#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_user_domain {
    use crate::{aggregate, aggregate::Root, event::Event};

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

        fn id(&self) -> &Self::Id {
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

    impl Root<User> for UserRoot {
        fn ctx(&self) -> &aggregate::Context<User> {
            &self.0
        }

        fn ctx_mut(&mut self) -> &mut aggregate::Context<User> {
            &mut self.0
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

            Ok(UserRoot::from(aggregate::Context::record_new(
                Event::from(UserEvent::WasCreated { email, password }),
            )?))
        }

        pub(crate) fn change_password(&mut self, password: String) -> Result<(), UserError> {
            if password.is_empty() {
                return Err(UserError::EmptyPassword);
            }

            self.ctx_mut()
                .record_that(Event::from(UserEvent::PasswordWasChanged { password }))?;

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
        event,
        event::Event,
        test,
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
            payload: Event::from(UserEvent::WasCreated { email, password }),
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
            payload: Event::from(UserEvent::PasswordWasChanged {
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
