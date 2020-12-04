//! Foundation traits for creating Domain abstractions
//! using [the `Aggregate` pattern](https://martinfowler.com/bliki/DDD_Aggregate.html).

use std::error::Error as StdError;
use std::fmt::{Debug, Display};

use async_trait::async_trait;

use futures::stream::TryStreamExt;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::eventstore::{EventStore, PersistedEvent, Select, Version};
use crate::{Event, Events};

/// A short extractor type for the Aggregate [`Id`].
///
/// [`Id`]: trait.Aggregate.html#associatedtype.Id
pub type IdOf<A> = <A as Aggregate>::Id;

/// An Aggregate manages a domain entity [`State`], acting as a _transaction boundary_.
///
/// It allows **state mutations** through the use of [`Command`]s, which the
/// Aggregate instance handles and emits a number of Domain [`Event`]s.
///
/// [`Event`]: trait.Aggregate.html#associatedtype.Event
/// [`State`]: trait.Aggregate.html#associatedtype.State
/// [`Command`]: trait.Aggregate.html#associatedtype.Command
#[async_trait]
pub trait Aggregate: Send + Sync {
    /// Aggregate identifier: this should represent an unique identifier to refer
    /// to a unique Aggregate instance.
    type Id: PartialEq + Send + Sync;

    /// State of the Aggregate: this should represent the Domain Entity data structure.
    type State: Default + Send + Sync;

    /// Represents a specific, domain-related change to the Aggregate [`State`].
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    type DomainEvent: Send + Sync;

    /// Commands are all the possible operations available on an Aggregate.
    /// Use Commands to model business use-cases or [`State`] mutations.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    type Command: Send + Sync;

    type ApplyError: StdError + Send + Sync;

    /// Possible failures while [`apply`]ing [`Event`]s or handling [`Command`]s.
    ///
    /// [`apply`]: trait.Aggregate.html#method.apply
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    type HandleError: StdError + Send + Sync;

    /// Handles the requested [`Command`] and returns a list of [`Event`]s
    /// to apply the [`State`] mutation based on the current representation of the State.
    ///
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    async fn handle(
        &mut self,
        state: &Self::State,
        command: Self::Command,
    ) -> Result<Events<Self::DomainEvent>, Self::HandleError>;

    /// Applies an [`Event`] to the current Aggregate [`State`].
    ///
    /// To enforce immutability, this method takes ownership of the previous [`State`]
    /// and the current [`Event`] to apply, and returns the new version of the [`State`]
    /// or an error.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    fn apply(
        state: Self::State,
        event: Event<Self::DomainEvent>,
    ) -> Result<Self::State, Self::ApplyError>;

    /// Applies a list of [`Event`]s from an `Iterator`
    /// to the current Aggregate [`State`].
    ///
    /// Useful to recreate the [`State`] of an Aggregate when the [`Event`]s
    /// are located in-memory.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    #[inline]
    fn fold<I>(state: Self::State, mut events: I) -> Result<Self::State, Self::ApplyError>
    where
        I: Iterator<Item = Event<Self::DomainEvent>>,
    {
        events.try_fold(state, Self::apply)
    }
}

/// Builder type for new [`AggregateRoot`] instances.
///
/// [`AggregateRoot`]: struct.AggregateRoot.html
#[derive(Clone)]
pub struct AggregateRootBuilder<T: Aggregate> {
    aggregate: T,
}

impl<T: Aggregate> From<T> for AggregateRootBuilder<T> {
    #[inline]
    fn from(aggregate: T) -> Self {
        Self { aggregate }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RehydrateError<A, ES>
where
    A: StdError + 'static,
    ES: StdError + 'static,
{
    #[error("rehydrate error, failed to apply event: {0}")]
    Apply(#[source] A),

    #[error("rehydrate error, failed to process next event: {0}")]
    EventStore(#[source] ES),
}

impl<T> AggregateRootBuilder<T>
where
    T: Aggregate + Clone + 'static,
{
    async fn rehydrate<Evts, E>(
        &self,
        id: T::Id,
        events: Evts,
    ) -> Result<AggregateRoot<T>, RehydrateError<T::ApplyError, E>>
    where
        E: StdError + 'static,
        Evts: futures::Stream<Item = Result<PersistedEvent<T::Id, T::DomainEvent>, E>>,
    {
        let (version, state): (u32, T::State) = events
            .map_err(RehydrateError::EventStore)
            .try_fold((0, T::State::default()), |(_, state), event| async move {
                let version = event.sequence_number();
                let next = T::apply(state, event.into()).map(|state| (version, state));

                Ok(next.map_err(RehydrateError::Apply)?)
            })
            .await?;

        Ok(AggregateRoot {
            id,
            version,
            aggregate: self.aggregate.clone(),
            state,
            uncommitted_events: Vec::new(),
        })
    }
}

/// An `AggregateRoot` represents an handler to the [`Aggregate`] it's managing,
/// such as:
///
/// * Owning its [`State`], [`Id`] and version,
/// * Proxying [`Command`]s to the [`Aggregate`] using the current [`State`],
/// * Keeping a list of [`Event`]s to commit after [`Command`] execution.
///
/// ## Initialize
///
/// An `AggregateRoot` can only be initialized using the [`AggregateRootBuilder`].
///
/// Check [`AggregateRootBuilder::build`] for more information.
///
/// [`Aggregate`]: trait.Aggregate.html
/// [`Id`]: trait.Aggregate.html@associatedtype.Id
/// [`Event`]: trait.Aggregate.html#associatedtype.Event
/// [`State`]: trait.Aggregate.html#associatedtype.State
/// [`Command`]: trait.Aggregate.html#associatedtype.Event
/// [`AggregateRootBuilder`]: struct.AggregateRootBuilder.html
/// [`AggregateRootBuilder::build`]: struct.AggregateRootBuilder.html#method.build
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AggregateRoot<T>
where
    T: Aggregate + 'static,
{
    id: T::Id,
    version: u32,

    #[cfg_attr(feature = "serde", serde(flatten))]
    state: T::State,

    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    aggregate: T,

    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    uncommitted_events: Events<T::DomainEvent>,
}

impl<T> PartialEq for AggregateRoot<T>
where
    T: Aggregate,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
{
    /// Returns a reference to the Aggregate [`Id`] that represents
    /// the entity wrapped by this [`AggregateRoot`] instance.
    ///
    /// [`Id`]: trait.Aggregate.html#associatedtype.Id
    /// [`AggregateRoot`]: struct.AggregateRoot.html
    #[inline]
    pub fn id(&self) -> &T::Id {
        &self.id
    }

    /// Takes the list of events to commit from the current instance,
    /// resetting it to `None`.
    #[inline]
    pub(crate) fn flush_events(&mut self) -> Events<T::DomainEvent> {
        std::mem::replace(&mut self.uncommitted_events, Vec::new())
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
    T::DomainEvent: Clone,
{
    fn apply(&mut self, events: Events<T::DomainEvent>) -> Result<(), T::ApplyError> {
        self.uncommitted_events.append(&mut events.clone());
        self.state = T::fold(std::mem::take(&mut self.state), events.into_iter())?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum AggregateRootError<Id, HandleError, ApplyError>
where
    Id: Display + Debug,
    HandleError: StdError + 'static,
    ApplyError: StdError + 'static,
{
    #[error("command refers to another aggregate instance with id: `{0}`")]
    MismatchId(Id),

    #[error("aggregate failed to process command: {0}")]
    Handle(#[source] HandleError),

    #[error("aggregate root failed to update its state: {0}")]
    Apply(#[source] ApplyError),
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
    T::Id: Display + Debug + Clone,
    T::DomainEvent: Clone,
    T::State: Clone,
    T::Command: AsRef<T::Id> + Debug,
{
    /// Handles the submitted [`Command`] using the [`Aggregate::handle`] method
    /// and updates the Aggregate [`State`].
    ///
    /// Returns a `&mut self` reference to allow for _method chaining_.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    /// [`Aggregate::handle`]: trait.Aggregate.html#method.handle
    #[cfg_attr(
        feature = "with-racing",
        tracing::instrument(level = "debug", name = "AggregateRoot::handle", skip(self))
    )]
    pub async fn handle(
        &mut self,
        command: T::Command,
    ) -> Result<(), AggregateRootError<T::Id, T::HandleError, T::ApplyError>> {
        if self.id() != command.as_ref() {
            return Err(AggregateRootError::MismatchId(command.as_ref().clone()));
        }

        let events = self
            .aggregate
            .handle(&self.state, command)
            .await
            .map_err(AggregateRootError::Handle)?;

        // Apply new events only if the command handling produced some.
        if !events.is_empty() {
            self.apply(events).map_err(AggregateRootError::Apply)?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Repository<A, ES>
where
    A: Aggregate,
    ES: EventStore<A::Id, A::DomainEvent>,
{
    aggregate_root_builder: AggregateRootBuilder<A>,
    event_store: ES,
}

impl<A, ES> Repository<A, ES>
where
    A: Aggregate,
    ES: EventStore<A::Id, A::DomainEvent>,
{
    #[inline]
    pub fn new(aggregate_root_builder: AggregateRootBuilder<A>, event_store: ES) -> Self {
        Repository {
            aggregate_root_builder,
            event_store,
        }
    }
}

impl<A, ES> Repository<A, ES>
where
    A: Aggregate + Clone + 'static,
    <A as Aggregate>::Id: Clone,
    <A as Aggregate>::ApplyError: StdError + 'static,
    ES: EventStore<A::Id, A::DomainEvent>,
    <ES as EventStore<A::Id, A::DomainEvent>>::StreamError: StdError + 'static,
{
    pub async fn get(
        &self,
        id: &A::Id,
    ) -> Result<AggregateRoot<A>, RehydrateError<A::ApplyError, ES::StreamError>> {
        let events = self.event_store.stream(id, Select::All);

        self.aggregate_root_builder
            .rehydrate(id.clone(), events)
            .await
    }

    pub async fn add(&mut self, root: &mut AggregateRoot<A>) -> Result<(), ES::AppendError> {
        let events_to_commit = root.flush_events();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let new_version = self
            .event_store
            .append(root.id(), Version::Exact(root.version), events_to_commit)
            .await?;

        root.version = new_version;

        Ok(())
    }
}
