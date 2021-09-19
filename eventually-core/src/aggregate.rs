//! Foundation traits for creating Domain abstractions
//! using [the `Aggregate` pattern](https://martinfowler.com/bliki/DDD_Aggregate.html).

use std::fmt::Debug;
use std::ops::Deref;

use async_trait::async_trait;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::versioning::Versioned;

/// A short extractor type for the [`Aggregate`] [`Id`](Aggregate::Id).
pub type AggregateId<A> = <A as Aggregate>::Id;

/// An [`Aggregate`] manages a domain entity [`State`](Aggregate::State), acting
/// as a _transaction boundary_.
///
/// It allows **state mutations** through the use of
/// [`Command`](Aggregate::Command)s, which the Aggregate instance handles and
/// emits a number of Domain [`Event`](Aggregate::Event)s.
#[async_trait]
pub trait Aggregate {
    /// Aggregate identifier: this should represent an unique identifier to
    /// refer to a unique Aggregate instance.
    type Id: Eq;

    /// State of the Aggregate: this should represent the Domain Entity data
    /// structure.
    type State: Default;

    /// Represents a specific, domain-related change to the Aggregate
    /// [`State`](Aggregate::State).
    type Event;

    /// Commands are all the possible operations available on an Aggregate.
    /// Use Commands to model business use-cases or [`State`](Aggregate::State)
    /// mutations.
    type Command;

    /// Possible failures while [`apply`](Aggregate::apply)ing
    /// [`Event`](Aggregate::Event)s or handling
    /// [`Command`](Aggregate::Command)s.
    type Error;

    /// Applies an [`Event`](Aggregate::Event) to the current Aggregate
    /// [`State`](Aggregate::State).
    ///
    /// To enforce immutability, this method takes ownership of the previous
    /// [`State`](Aggregate::State) and the current
    /// [`Event`](Aggregate::Event) to apply, and returns the new version of the
    /// [`State`](Aggregate::State) or an error.
    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;

    /// Handles the requested [`Command`](Aggregate::Command) and returns a list
    /// of [`Event`](Aggregate::Event)s to apply the
    /// [`State`](Aggregate::State) mutation based on the current representation
    /// of the State.
    async fn handle(
        &self,
        id: &Self::Id,
        state: &Self::State,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error>;
}

/// Extension trait with some handy methods to use with [`Aggregate`]s.
pub trait AggregateExt: Aggregate {
    /// Applies a list of [`Event`](Aggregate::Event)s from an `Iterator`
    /// to the current Aggregate [`State`](Aggregate::State).
    ///
    /// Useful to recreate the [`State`](Aggregate::State) of an Aggregate when
    /// the [`Event`](Aggregate::Event)s are located in-memory.
    #[inline]
    fn fold<I>(state: Self::State, mut events: I) -> Result<Self::State, Self::Error>
    where
        I: Iterator<Item = Self::Event>,
    {
        events.try_fold(state, Self::apply)
    }
}

impl<T> AggregateExt for T where T: Aggregate {}

/// Builder type for new [`AggregateRoot`] instances.
#[derive(Clone)]
pub struct AggregateRootBuilder<T>
where
    T: Aggregate,
{
    aggregate: T,
}

impl<T> From<T> for AggregateRootBuilder<T>
where
    T: Aggregate,
{
    #[inline]
    fn from(aggregate: T) -> Self {
        Self { aggregate }
    }
}

impl<T> AggregateRootBuilder<T>
where
    T: Aggregate + Clone,
{
    /// Builds a new [`AggregateRoot`] instance for the specified [`Aggregate`]
    /// [`Id`](Aggregate::Id).
    #[inline]
    pub fn build(&self, id: T::Id) -> AggregateRoot<T> {
        self.build_with_state(id, 0, Default::default())
    }

    /// Builds a new [`AggregateRoot`] instance for the specified Aggregate
    /// with a specified [`State`](Aggregate::State) value.
    #[inline]
    pub fn build_with_state(&self, id: T::Id, version: u32, state: T::State) -> AggregateRoot<T> {
        AggregateRoot {
            id,
            version,
            state,
            aggregate: self.aggregate.clone(),
            to_commit: None,
        }
    }
}

/// An [`AggregateRoot`] represents an handler to the [`Aggregate`] it's
/// managing, such as:
///
/// * Owning its [`State`](Aggregate::State), [`Id`](Aggregate::Id) and version,
/// * Proxying [`Command`](Aggregate::Command)s to the [`Aggregate`] using the
///   current [`State`](Aggregate::State),
/// * Keeping a list of [`Event`](Aggregate::Event)s to commit after
///   [`Command`](Aggregate::Command) execution.
///
/// ## Initialize
///
/// An [`AggregateRoot`] can only be initialized using the
/// [`AggregateRootBuilder`].
///
/// Check [`AggregateRootBuilder::build`] for more information.

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
    to_commit: Option<Vec<T::Event>>,
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

impl<T> Versioned for AggregateRoot<T>
where
    T: Aggregate,
{
    #[inline]
    fn version(&self) -> u32 {
        self.version
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
{
    /// Returns a reference to the Aggregate [`Id`](Aggregate::Id) that
    /// represents the entity wrapped by this [`AggregateRoot`] instance.
    #[inline]
    pub fn id(&self) -> &T::Id {
        &self.id
    }

    /// Returns a reference to the current Aggregate
    /// [`State`](Aggregate::State).
    #[inline]
    pub fn state(&self) -> &T::State {
        &self.state
    }

    /// Takes the list of events to commit from the current instance,
    /// resetting it to `None`.
    #[inline]
    pub(crate) fn take_events_to_commit(&mut self) -> Option<Vec<T::Event>> {
        std::mem::replace(&mut self.to_commit, None)
    }

    /// Returns a new [`AggregateRoot`] having the specified version.
    #[inline]
    pub(crate) fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
}

impl<T> Deref for AggregateRoot<T>
where
    T: Aggregate,
{
    type Target = T::State;

    fn deref(&self) -> &Self::Target {
        self.state()
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
    T::Event: Clone,
    T::State: Clone,
    T::Command: Debug,
{
    /// Handles the submitted [`Command`](Aggregate::Command) using the
    /// [`Aggregate::handle`] method and updates the Aggregate
    /// [`State`](Aggregate::State).
    ///
    /// Returns a `&mut self` reference to allow for _method chaining_.
    #[cfg_attr(
        feature = "with-tracing",
        tracing::instrument(level = "debug", name = "AggregateRoot::handle", skip(self))
    )]
    pub async fn handle(&mut self, command: T::Command) -> Result<&mut Self, T::Error> {
        let mut events = self
            .aggregate
            .handle(self.id(), self.state(), command)
            .await?;

        // Only apply new events if the command handling actually
        // produced new ones.
        self.state = T::fold(self.state.clone(), events.clone().into_iter())?;
        self.to_commit = Some(match self.to_commit.take() {
            None => events,
            Some(mut list) => {
                list.append(&mut events);
                list
            }
        });

        Ok(self)
    }
}
