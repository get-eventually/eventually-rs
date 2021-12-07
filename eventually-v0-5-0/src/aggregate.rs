use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use futures::TryStreamExt;

use crate::{event, event::Event, version::Version};

pub trait Aggregate: Sized + Send + Sync {
    type Id: Send + Sync;
    type Event: Send + Sync;
    type Error: Send + Sync;

    fn id(&self) -> &Self::Id;

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error>;
}

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
    pub fn aggregate(&self) -> &T {
        &self.aggregate
    }

    fn take_uncommitted_events(&mut self) -> Vec<Event<T::Event>> {
        std::mem::take(&mut self.recorded_events)
    }

    fn rehydrate_from(event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.payload)?,
            recorded_events: Vec::default(),
        })
    }

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
    pub fn record_new(event: Event<T::Event>) -> Result<Context<T>, T::Error> {
        Ok(Context {
            version: 1,
            aggregate: T::apply(None, event.payload.clone())?,
            recorded_events: vec![event],
        })
    }

    pub fn record_that(&mut self, event: Event<T::Event>) -> Result<(), T::Error> {
        self.aggregate = T::apply(Some(self.aggregate.clone()), event.payload.clone())?;
        self.recorded_events.push(event);
        self.version += 1;

        Ok(())
    }
}

pub trait Root<T>: From<Context<T>> + Send + Sync
where
    T: Aggregate,
{
    fn ctx(&self) -> &Context<T>;

    fn ctx_mut(&mut self) -> &mut Context<T>;

    fn aggregate_id<'a>(&'a self) -> &'a T::Id
    where
        T: 'a,
    {
        self.ctx().aggregate().id()
    }
}

#[async_trait]
pub trait Repository<T, R>: Send + Sync
where
    T: Aggregate,
    R: Root<T>,
{
    type Error;

    async fn get(&self, id: &T::Id) -> Result<R, Self::Error>;

    async fn store(&self, root: &mut R) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum EventSourcedRepositoryError<E, SE, AE> {
    #[error("aggregate root was not found")]
    AggregateRootNotFound,

    #[error("failed to rehydrate aggregate from event stream: {}", 0)]
    RehydrateAggregate(#[source] E),

    #[error("event store failed while streaming events: {}", 0)]
    StreamFromStore(#[source] SE),

    #[error("event store failed while appending events: {}", 0)]
    AppendToStore(#[source] AE),
}

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

#[allow(clippy::semicolon_if_nothing_returned)] // False positives :shrugs:
#[cfg(test)]
mod test {
    use crate::{
        aggregate,
        aggregate::{Repository, Root},
        event,
        event::Event,
        test,
        test::store::EventStoreExt,
    };

    #[derive(Debug, Clone)]
    struct User {
        email: String,
        password: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum UserEvent {
        WasCreated { email: String, password: String },
        PasswordWasChanged { password: String },
    }

    #[derive(Debug, thiserror::Error)]
    enum UserError {
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
    struct UserRoot(aggregate::Context<User>);

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
        fn create(email: String, password: String) -> Result<Self, UserError> {
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

        fn change_password(&mut self, password: String) -> Result<(), UserError> {
            if password.is_empty() {
                return Err(UserError::EmptyPassword);
            }

            self.ctx_mut()
                .record_that(Event::from(UserEvent::PasswordWasChanged { password }))?;

            Ok(())
        }
    }

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
}
