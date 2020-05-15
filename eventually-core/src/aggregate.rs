//! Foundation traits for creating Domain abstractions
//! using [the `Aggregate` pattern](https://martinfowler.com/bliki/DDD_Aggregate.html).

use std::sync::Arc;

use futures::future::BoxFuture;

/// A trait for data structures that can be identified by an id.
pub trait Identifiable {
    /// Type of the data id.
    /// An id must support total equality.
    type Id: Eq;

    /// Data structure id accessor.
    fn id(&self) -> Self::Id;
}

impl<State> Identifiable for Option<State>
where
    State: Identifiable,
    State::Id: Default,
{
    type Id = State::Id;

    #[inline]
    fn id(&self) -> Self::Id {
        self.as_ref().map_or_else(Self::Id::default, |id| id.id())
    }
}

/// A short extractor type for the Aggregate id, found in the Aggregate [`State`].
///
/// [`State`]: trait.Aggregate.html#associatedtype.State
pub type AggregateId<A> = <<A as Aggregate>::State as Identifiable>::Id;

/// An Aggregate manages a domain entity [`State`], acting as a _transaction boundary_.
///
/// It allows **state mutations** through the use of [`Command`]s, which the
/// Aggregate instance handles and emits a number of Domain [`Event`]s.
///
/// [`Event`]: trait.Aggregate.html#associatedtype.Event
/// [`State`]: trait.Aggregate.html#associatedtype.State
/// [`Command`]: trait.Aggregate.html#associatedtype.Command
pub trait Aggregate {
    /// State of the Aggregate: this should represent the Domain Entity data structure.
    type State: Identifiable;

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
        state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized;
}

impl<T> Aggregate for Arc<T>
where
    T: Aggregate,
{
    type State = T::State;
    type Event = T::Event;
    type Command = T::Command;
    type Error = T::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        T::apply(state, event)
    }

    fn handle<'agg, 'st: 'agg>(
        &'agg self,
        state: &'st Self::State,
        command: Self::Command,
    ) -> BoxFuture<'agg, Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized,
    {
        T::handle(self, state, command)
    }
}

/// Extension trait with some handy methods to use with [`Aggregate`]s.
///
/// [`Aggregate`]: trait.Aggregate.html
pub trait AggregateExt: Aggregate {
    /// Constructs a new, empty [`AggregateRoot`] using the current [`Aggregate`].
    ///
    /// [`Aggregate`]: trait.Aggregate.html
    /// [`AggregateRoot`]: struct.AggregateRoot.html
    #[inline]
    fn root(&self) -> AggregateRoot<Self>
    where
        Self: Sized + Clone,
        Self::State: Default,
    {
        AggregateRoot::from(self.clone())
    }

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

/// An `AggregateRoot` represents an handler to the [`Aggregate`] it's managing,
/// such as:
///
/// * Owning the current, local Aggregate [`State`],
/// * Proxying [`Command`]s to the [`Aggregate`] using the current [`State`],
/// * Keeping a list of [`Event`]s to commit after [`Command`] execution.
///
/// ## Initialize
///
/// `AggregateRoot` can be initialized in two ways:
///
/// 1. Using the `From<Aggregate>` method:
///     ```text
///     let root = AggregateRoot::from(aggregate);
///     ```
///
/// 2. Using the [`AggregateExt`] extension trait to call [`root()`] on the [`Aggregate`]
/// instance:
///     ```text
///     // This will result in a Clone of the Aggregate instance.
///     let root = aggregate.root();
///     ```
///
/// [`Aggregate`]: trait.Aggregate.html
/// [`AggregateExt`]: trait.AggregateExt.html
/// [`root()`]: trait.AggregateExt.html#method.root
/// [`Event`]: trait.Aggregate.html#associatedtype.Event
/// [`State`]: trait.Aggregate.html#associatedtype.State
/// [`Command`]: trait.Aggregate.html#associatedtype.Event
#[derive(Debug)]
pub struct AggregateRoot<T>
where
    T: Aggregate + 'static,
{
    pub(crate) state: T::State,
    aggregate: T,
    pub(crate) to_commit: Option<Vec<T::Event>>,
}

impl<T> Identifiable for AggregateRoot<T>
where
    T: Aggregate,
{
    type Id = AggregateId<T>;

    #[inline]
    fn id(&self) -> Self::Id {
        self.state.id()
    }
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

impl<T> From<T> for AggregateRoot<T>
where
    T: Aggregate,
    T::State: Default,
{
    #[inline]
    fn from(aggregate: T) -> Self {
        Self::new(aggregate, Default::default())
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
{
    /// Returns a reference to the current Aggregate [`State`].
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    #[inline]
    pub fn state(&self) -> &T::State {
        &self.state
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
{
    /// Creates a new `AggregateRoot` instance wrapping the specified [`State`].
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    #[inline]
    pub fn new(aggregate: T, state: T::State) -> Self {
        Self {
            state,
            aggregate,
            to_commit: None,
        }
    }
}

impl<T> AggregateRoot<T>
where
    T: AggregateExt,
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
        let mut events = self.aggregate.handle(self.state(), command).await?;

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
