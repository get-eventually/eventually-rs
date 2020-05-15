//! Contains a different flavour of the [`Aggregate`] trait,
//! while still maintaining compatibility through [`AsAggregate`] type.
//!
//! Check out [`optional::Aggregate`] for more information.
//!
//! [`Aggregate`]: ../../eventually_core/aggregate/trait.Aggregate.html
//! [`AsAggregate`]: struct.AsAggregate.html
//! [`optional::Aggregate`]: trait.Aggregate.html

use eventually_core::aggregate::Identifiable;

use futures::future::BoxFuture;

/// An `Option`-flavoured, [`Aggregate`]-compatible trait
/// to model Aggregates having an optional [`State`].
///
/// Use [`as_aggregate`] to get an [`Aggregate`]-compatible instance
/// of this trait.
///
/// [`Aggregate`]: ../../eventually_core/aggregate/trait.Aggregate.html
/// [`State`]: ../../eventually_core/aggregate/trait.Aggregate.html#associatedtype.State
/// [`as_aggregate`]: trait.Aggregate.html#method.as_aggregate
pub trait Aggregate {
    /// State of the Aggregate.
    ///
    /// Check out [`Aggregate::State`] for more information.
    ///
    /// [`Aggregate::State`]: ../../eventually_core/aggregate/trait.Aggregate.html#associatedtype.State
    type State: Identifiable;

    /// Events produced and supported by the Aggregate.
    ///
    /// Check out [`Aggregate::Event`] for more information.
    ///
    /// [`Aggregate::Event`]: ../../eventually_core/aggregate/trait.Aggregate.html#associatedtype.Event
    type Event;

    /// Commands supported by the Aggregate.
    ///
    /// Check out [`Aggregate::Command`] for more information.
    ///
    /// [`Aggregate::Command`]: ../../eventually_core/aggregate/trait.Aggregate.html#associatedtype.Command
    type Command;

    /// Error produced by the the Aggregate while applying [`Event`]s
    /// or handling [`Command`]s.
    ///
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    type Error;

    /// Applies the specified [`Event`] when the [`State`] is empty.
    ///
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    fn apply_first(event: Self::Event) -> Result<Self::State, Self::Error>;

    /// Applies the specified [`Event`] on a pre-existing [`State`] value.
    ///
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    fn apply_next(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;

    /// Handles the specified [`Command`] when the [`State`] is empty.
    ///
    /// [`Command`]: trait.Aggregate.html#associatedtype.Command
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    fn handle_first(
        &self,
        command: Self::Command,
    ) -> BoxFuture<Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized;

    /// Handles the specified [`Command`] on a pre-existing [`State`] value.
    ///
    /// [`Command`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    fn handle_next<'a, 's: 'a>(
        &'a self,
        state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized;

    /// Translates the current [`optional::Aggregate`] instance into
    /// a _newtype instance_ compatible with the core [`Aggregate`] trait.
    ///
    /// [`optional::Aggregate`]: trait.Aggregate.html
    /// [`Aggregate`]: ../../eventually_core/aggregate/trait.Aggregate.html
    #[inline]
    fn as_aggregate(self) -> AsAggregate<Self>
    where
        Self: Sized,
    {
        AsAggregate::from(self)
    }
}

/// _Newtype pattern_ to ensure compatibility between [`optional::Aggregate`] trait
/// and the core [`Aggregate`] trait.
///
/// ## Usage
///
/// 1. Use `From<Aggregate>` trait implementation:
///     ```text
///     use eventually_util::optional::AsAggregate;
///
///     let aggregate = AsAggregate::from(MyOptionalAggregate);
///     ```
/// 2. Use the [`Aggregate::as_aggregate`] method:
///     ```text
///     let aggregate = MyOptionalAggregate.as_aggregate();
///     ```
///
/// [`optional::Aggregate`]: trait.Aggregate.html
/// [`Aggregate::as_aggregate`]: trait.Aggregate.html#method.as_aggregate
/// [`Aggregate`]: ../../eventually_core/aggregate/trait.Aggregate.html
#[derive(Debug, Clone)]
pub struct AsAggregate<A>(A);

impl<A> From<A> for AsAggregate<A> {
    #[inline]
    fn from(value: A) -> Self {
        AsAggregate(value)
    }
}

impl<A> eventually_core::aggregate::Aggregate for AsAggregate<A>
where
    A: Aggregate,
    A: Send + Sync,
    A::Command: Send + Sync,
    A::State: Identifiable + Send + Sync,
    <A::State as Identifiable>::Id: Default,
{
    type State = Option<A::State>;
    type Event = A::Event;
    type Command = A::Command;
    type Error = A::Error;

    #[inline]
    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        match state {
            None => A::apply_first(event).map(Some),
            Some(state) => A::apply_next(state, event).map(Some),
        }
    }

    fn handle<'a, 's: 'a>(
        &'a self,
        state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized,
    {
        Box::pin(match state {
            None => self.0.handle_first(command),
            Some(state) => self.0.handle_next(state, command),
        })
    }
}
