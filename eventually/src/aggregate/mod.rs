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

use crate::version::Version;
use crate::{event, message};

pub mod repository;
pub mod test;

use futures::TryStreamExt;
pub use repository::{EventSourced as EventSourcedRepository, Repository};

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
    type Id: Send + Sync;

    /// The type of Domain Events that interest this Aggregate.
    /// Usually, this type should be an `enum`.
    type Event: message::Message + Send + Sync + Clone;

    /// The error type that can be returned by [`Aggregate::apply`] when
    /// mutating the Aggregate state.
    type Error: Send + Sync;

    /// A unique name identifier for this Aggregate type.
    fn type_name() -> &'static str;

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

/// An Aggregate Root represents the Domain Entity object used to
/// load and save an [Aggregate] from and to a [Repository], and
/// to perform actions that may result in new Domain Events
/// to change the state of the Aggregate.
///
/// The Aggregate state and list of Domain Events recorded
/// are handled by the [Root] object itself.
///
/// ```text
/// #[derive(Debug, Clone)]
/// struct MyAggregate {
///     // Here goes the state of the Aggregate.
/// };
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// enum MyAggregateEvent {
///     // Here we list the Domain Events for the Aggregate.
///     EventHasHappened,
/// }
///
/// impl Aggregate for MyAggregate {
///     type Id = i64; // Just for the sake of the example.
///     type Event = MyAggregateEvent;
///     type Error = (); // Just for the sake of the example. Use a proper error here.
///
///     fn aggregate_id(&self) -> &Self::Id {
///         todo!()
///     }
///
///     fn apply(this: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
///         todo!()
///     }
/// }
///
/// // This type is necessary in order to create a new vtable
/// // for the method implementations in the block below.
/// #[derive(Debug, Clone)]
/// struct MyAggregateRoot(Root<MyAggregate>)
///
/// impl MyAggregateRoot {
///     pub fn do_something() -> Result<MyAggregate, ()> {
///         // Here, we record a new Domain Event through the Root<MyAggregate> object.
///         //
///         // This will record the new Domain Event in a list of events to commit,
///         // and call the `MyAggregate::apply` method to create the Aggregate state.
///         Root::<MyAggregate>::record_new(MyAggregateEvent::EventHasHappened)
///             .map(MyAggregateRoot)
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct Root<T>
where
    T: Aggregate,
{
    aggregate: T,
    version: Version,
    recorded_events: Vec<event::Envelope<T::Event>>,
}

impl<T> std::ops::Deref for Root<T>
where
    T: Aggregate,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.aggregate
    }
}

impl<T> Root<T>
where
    T: Aggregate,
{
    /// Returns the current version for the [Aggregate].
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the unique identifier of the [Aggregate].
    pub fn aggregate_id(&self) -> &T::Id {
        self.aggregate.aggregate_id()
    }

    /// Maps the [Aggregate] value contained within [Root]
    /// to a different type, that can be converted through [From] trait.
    ///
    /// Useful to convert an [Aggregate] type to a data transfer object to use
    /// for database storage.
    pub fn to_aggregate_type<K>(&self) -> K
    where
        K: From<T>,
    {
        K::from(self.aggregate.clone())
    }

    /// Returns the list of uncommitted, recorded Domain [Event]s from the [Root]
    /// and resets the internal list to its default value.
    #[doc(hidden)]
    pub fn take_uncommitted_events(&mut self) -> Vec<event::Envelope<T::Event>> {
        std::mem::take(&mut self.recorded_events)
    }

    /// Creates a new [Aggregate] [Root] instance by applying the specified
    /// Domain Event.
    ///
    /// Example of usage:
    /// ```text
    /// use eventually::{
    ///     event,
    ///     aggregate::Root,
    ///     aggregate,
    /// };
    ///
    /// let my_aggregate_root = MyAggregateRoot::record_new(
    ///     event::Envelope::from(MyDomainEvent { /* something */ })
    ///  )?;
    /// ```
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    pub fn record_new(event: event::Envelope<T::Event>) -> Result<Self, T::Error> {
        Ok(Root {
            version: 1,
            aggregate: T::apply(None, event.message.clone())?,
            recorded_events: vec![event],
        })
    }

    /// Records a change to the [Aggregate] [Root], expressed by the specified
    /// Domain Event.
    ///
    /// Example of usage:
    /// ```text
    /// use eventually::{
    ///     event,
    ///     aggregate::Root,
    /// };
    ///
    /// impl MyAggregateRoot {
    ///     pub fn update_name(&mut self, name: String) -> Result<(), MyAggregateError> {
    ///         if name.is_empty() {
    ///             return Err(MyAggregateError::NameIsEmpty);
    ///         }
    ///
    ///         self.record_that(
    ///             event::Envelope::from(MyAggergateEvent::NameWasChanged { name })
    ///         )
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    pub fn record_that(&mut self, event: event::Envelope<T::Event>) -> Result<(), T::Error> {
        self.aggregate = T::apply(Some(self.aggregate.clone()), event.message.clone())?;
        self.recorded_events.push(event);
        self.version += 1;

        Ok(())
    }
}

/// List of possible errors that can be returned by [`Root::rehydrate_async`].
#[derive(Debug, thiserror::Error)]
pub enum RehydrateError<T, I> {
    /// Error returned during rehydration when the [Aggregate Root][Root]
    /// is applying a Domain Event using [`Aggregate::apply`].
    ///
    /// This usually implies the Event Stream for the [Aggregate]
    /// contains corrupted or unexpected data.
    #[error("failed to apply domain event while rehydrating aggregate: {0}")]
    Domain(#[source] T),

    /// This error is returned by [`Root::rehydrate_async`] when the underlying
    /// [`futures::TryStream`] has returned an error.
    #[error("failed to rehydrate aggregate from event stream: {0}")]
    Inner(#[source] I),
}

impl<T> Root<T>
where
    T: Aggregate,
{
    /// Rehydrates an [Aggregate] Root from its state and version.
    /// Useful for [Repository] implementations outside the [EventSourcedRepository] one.
    #[doc(hidden)]
    pub fn rehydrate_from_state(version: Version, aggregate: T) -> Root<T> {
        Root {
            version,
            aggregate,
            recorded_events: Vec::default(),
        }
    }

    /// Rehydrates an [Aggregate Root][Root] from a stream of Domain Events.
    #[doc(hidden)]
    pub(crate) fn rehydrate(
        mut stream: impl Iterator<Item = event::Envelope<T::Event>>,
    ) -> Result<Option<Root<T>>, T::Error> {
        stream.try_fold(None, |ctx: Option<Root<T>>, event| {
            let new_ctx_result = match ctx {
                None => Root::<T>::rehydrate_from(event),
                Some(ctx) => ctx.apply_rehydrated_event(event),
            };

            Ok(Some(new_ctx_result?))
        })
    }

    /// Rehydrates an [Aggregate Root][Root] from a stream of Domain Events.
    #[doc(hidden)]
    pub(crate) async fn rehydrate_async<Err>(
        stream: impl futures::TryStream<Ok = event::Envelope<T::Event>, Error = Err>,
    ) -> Result<Option<Root<T>>, RehydrateError<T::Error, Err>> {
        stream
            .map_err(RehydrateError::Inner)
            .try_fold(None, |ctx: Option<Root<T>>, event| async {
                let new_ctx_result = match ctx {
                    None => Root::<T>::rehydrate_from(event),
                    Some(ctx) => ctx.apply_rehydrated_event(event),
                };

                Ok(Some(new_ctx_result.map_err(RehydrateError::Domain)?))
            })
            .await
    }

    /// Creates a new [Root] instance from a Domain [Event]
    /// while rehydrating an [Aggregate].
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    #[doc(hidden)]
    pub(crate) fn rehydrate_from(event: event::Envelope<T::Event>) -> Result<Root<T>, T::Error> {
        Ok(Root {
            version: 1,
            aggregate: T::apply(None, event.message)?,
            recorded_events: Vec::default(),
        })
    }

    /// Applies a new Domain [Event] to the [Root] while rehydrating
    /// an [Aggregate].
    ///
    /// # Errors
    ///
    /// The method can return an error if the event to apply is unexpected
    /// given the current state of the Aggregate.
    #[doc(hidden)]
    pub(crate) fn apply_rehydrated_event(
        mut self,
        event: event::Envelope<T::Event>,
    ) -> Result<Root<T>, T::Error> {
        self.aggregate = T::apply(Some(self.aggregate), event.message)?;
        self.version += 1;

        Ok(self)
    }
}

// The warnings are happening due to usage of the methods only inside #[cfg(test)]
#[allow(dead_code)]
#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_user_domain {
    use crate::{aggregate, message};

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

        fn type_name() -> &'static str {
            "User"
        }

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
                    },
                    UserEvent::WasCreated { .. } => Err(UserError::AlreadyCreated),
                },
            }
        }
    }

    impl aggregate::Root<User> {
        pub(crate) fn create(email: String, password: String) -> Result<Self, UserError> {
            if email.is_empty() {
                return Err(UserError::EmptyEmail);
            }

            if password.is_empty() {
                return Err(UserError::EmptyPassword);
            }

            Self::record_new(UserEvent::WasCreated { email, password }.into())
        }

        pub(crate) fn change_password(&mut self, password: String) -> Result<(), UserError> {
            if password.is_empty() {
                return Err(UserError::EmptyPassword);
            }

            self.record_that(UserEvent::PasswordWasChanged { password }.into())?;

            Ok(())
        }
    }
}

#[allow(clippy::semicolon_if_nothing_returned)] // False positives :shrugs:
#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::aggregate::repository::{Getter, Saver};
    use crate::aggregate::test_user_domain::{User, UserEvent};
    use crate::event::store::EventStoreExt;
    use crate::{aggregate, event, version};

    #[tokio::test]
    async fn repository_persists_new_aggregate_root() {
        let event_store = event::store::InMemory::<String, UserEvent>::default();
        let tracking_event_store = event_store.with_recorded_events_tracking();
        let user_repository =
            aggregate::EventSourcedRepository::<User, _>::from(tracking_event_store.clone());

        let email = "test@email.com".to_owned();
        let password = "not-a-secret".to_owned();

        let mut user = aggregate::Root::<User>::create(email.clone(), password.clone())
            .expect("user should be created successfully");

        user_repository
            .save(&mut user)
            .await
            .expect("user should be saved successfully");

        let expected_events = vec![event::Persisted {
            stream_id: email.clone(),
            version: 1,
            event: event::Envelope::from(UserEvent::WasCreated { email, password }),
        }];

        assert_eq!(expected_events, tracking_event_store.recorded_events());
    }

    #[tokio::test]
    async fn repository_retrieves_the_aggregate_root_and_stores_new_events() {
        let event_store = event::store::InMemory::<String, UserEvent>::default();
        let tracking_event_store = event_store.with_recorded_events_tracking();
        let user_repository =
            aggregate::EventSourcedRepository::<User, _>::from(tracking_event_store.clone());

        let email = "test@email.com".to_owned();
        let password = "not-a-secret".to_owned();

        let mut user = aggregate::Root::<User>::create(email.clone(), password.clone())
            .expect("user should be created successfully");

        user_repository
            .save(&mut user)
            .await
            .expect("user should be saved successfully");

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
            .save(&mut user)
            .await
            .expect("new user version should be saved successfully");

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
        let event_store = event::store::InMemory::<String, UserEvent>::default();
        let user_repository =
            aggregate::EventSourcedRepository::<User, _>::from(event_store.clone());

        let email = "test@email.com".to_owned();
        let password = "not-a-secret".to_owned();

        let mut user = aggregate::Root::<User>::create(email.clone(), password.clone())
            .expect("user should be created successfully");

        // We need to clone the User Aggregate Root instance to get the list
        // of uncommitted events from the Root context twice.
        let mut cloned_user = user.clone();

        // Saving the first User to the Repository.
        user_repository
            .save(&mut user)
            .await
            .expect("user should be saved successfully");

        // Simulating data race by duplicating the call to the Repository
        // with the same UserRoot instance that has already been committeed.
        let error = user_repository.save(&mut cloned_user).await.expect_err(
            "the repository should fail on the second .save() call with the cloned user",
        );

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
