//! Foundation traits for creating Domain abstractions
//! using [the `Aggregate` pattern](https://martinfowler.com/bliki/DDD_Aggregate.html).

use std::sync::Arc;

use futures::future::BoxFuture;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::versioning::Versioned;

/// A short extractor type for the Aggregate [`Id`].
///
/// [`Id`]: trait.Aggregate.html#associatedtype.Id
pub type AggregateId<A> = <A as Aggregate>::Id;

/// An Aggregate manages a domain entity [`State`], acting as a _transaction boundary_.
///
/// It allows **state mutations** through the use of [`Command`]s, which the
/// Aggregate instance handles and emits a number of Domain [`Event`]s.
///
/// [`Event`]: trait.Aggregate.html#associatedtype.Event
/// [`State`]: trait.Aggregate.html#associatedtype.State
/// [`Command`]: trait.Aggregate.html#associatedtype.Command
pub trait Aggregate {
    /// Aggregate identifier: this should represent an unique identifier to refer
    /// to a unique Aggregate instance.
    type Id: Eq;

    /// State of the Aggregate: this should represent the Domain Entity data structure.
    type State: Default;

    /// Represents a specific, domain-related change to the Aggregate [`State`].
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    type Event;

    /// Commands are all the possible operations available on an Aggregate.
    /// Use Commands to model business use-cases or [`State`] mutations.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    type Command;

    /// Possible failures while [`apply`]ing [`Event`]s or handling [`Command`]s.
    ///
    /// [`apply`]: trait.Aggregate.html#method.apply
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    type Error;

    /// Applies an [`Event`] to the current Aggregate [`State`].
    ///
    /// To enforce immutability, this method takes ownership of the previous [`State`]
    /// and the current [`Event`] to apply, and returns the new version of the [`State`]
    /// or an error.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;

    /// Handles the requested [`Command`] and returns a list of [`Event`]s
    /// to apply the [`State`] mutation based on the current representation of the State.
    ///
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    fn handle<'a, 's: 'a>(
        &'a self,
        id: &'s Self::Id,
        state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Option<Vec<Self::Event>>, Self::Error>>
    where
        Self: Sized;
}

/// Extension trait with some handy methods to use with [`Aggregate`]s.
///
/// [`Aggregate`]: trait.Aggregate.html
pub trait AggregateExt: Aggregate {
    /// Applies a list of [`Event`]s from an `Iterator`
    /// to the current Aggregate [`State`].
    ///
    /// Useful to recreate the [`State`] of an Aggregate when the [`Event`]s
    /// are located in-memory.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
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
///
/// [`AggregateRoot`]: struct.AggregateRoot.html
#[derive(Clone)]
pub struct AggregateRootBuilder<T> {
    aggregate: Arc<T>,
}

impl<T> From<Arc<T>> for AggregateRootBuilder<T>
where
    T: Aggregate,
{
    #[inline]
    fn from(aggregate: Arc<T>) -> Self {
        Self { aggregate }
    }
}

impl<T> AggregateRootBuilder<T>
where
    T: Aggregate,
{
    /// Builds a new [`AggregateRoot`] instance for the specified Aggregate [`Id`].
    ///
    /// [`Id`]: trait.Aggregate.html#associatedtype.Id
    /// [`AggregateRoot`]: struct.AggregateRoot.html
    #[inline]
    pub fn build(&self, id: T::Id) -> AggregateRoot<T> {
        self.build_with_state(id, 0, Default::default())
    }

    /// Builds a new [`AggregateRoot`] instance for the specified Aggregate
    /// with a specified [`State`] value.
    ///
    /// [`AggregateRoot`]: struct.AggregateRoot.html
    /// [`State`]: trait.Aggregate.html#associatedtype.State
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
/// [`AggregateExt`]: trait.AggregateExt.html
/// [`root()`]: trait.AggregateExt.html#method.root
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
    aggregate: Arc<T>,

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
    /// Returns a reference to the Aggregate [`Id`] that represents
    /// the entity wrapped by this [`AggregateRoot`] instance.
    ///
    /// [`Id`]: trait.Aggregate.html#associatedtype.Id
    /// [`AggregateRoot`]: struct.AggregateRoot.html
    #[inline]
    pub fn id(&self) -> &T::Id {
        &self.id
    }

    /// Returns a reference to the current Aggregate [`State`].
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
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

    /// Returns a new `AggregateRoot` having the specified version.
    #[inline]
    pub(crate) fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
    T::Event: Clone,
    T::State: Clone,
{
    /// Handles the submitted [`Command`] using the [`Aggregate::handle`] method
    /// and updates the Aggregate [`State`].
    ///
    /// Returns a `&mut self` reference to allow for _method chaining_.
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    /// [`Aggregate::handle`]: trait.Aggregate.html#method.handle
    pub async fn handle(&mut self, command: T::Command) -> Result<&mut Self, T::Error> {
        let events = self
            .aggregate
            .handle(self.id(), self.state(), command)
            .await?;

        // Only apply new events if the command handling actually
        // produced new ones.
        if let Some(mut events) = events {
            self.state = T::fold(self.state.clone(), events.clone().into_iter())?;
            self.to_commit = Some(match self.to_commit.take() {
                None => events,
                Some(mut list) => {
                    list.append(&mut events);
                    list
                }
            });
        }

        Ok(self)
    }
}
