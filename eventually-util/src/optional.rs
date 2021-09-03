//! Contains a different flavour of the [`Aggregate`] trait,
//! while still maintaining compatibility through [`AsAggregate`] type.
//!
//! Check out [`optional::Aggregate`](Aggregate) for more information.

use futures::future::BoxFuture;

/// An [`Option`]-flavoured, [`Aggregate`]-compatible trait
/// to model Aggregates having an optional [`State`](Aggregate::State).
///
/// Use [`as_aggregate`](Aggregate::as_aggregate) to get an
/// [`Aggregate`]-compatible instance of this trait.
pub trait Aggregate {
    /// Identifier type of the Aggregate.
    ///
    /// Check out [`Aggregate::Id`] for more information.
    type Id: Eq;

    /// State of the Aggregate.
    ///
    /// Check out [`Aggregate::State`] for more information.
    type State;

    /// Events produced and supported by the Aggregate.
    ///
    /// Check out [`Aggregate::Event`] for more information.
    type Event;

    /// Commands supported by the Aggregate.
    ///
    /// Check out [`Aggregate::Command`] for more information.
    type Command;

    /// Error produced by the the Aggregate while applying
    /// [`Event`](Aggregate::Event)s or handling
    /// [`Command`](Aggregate::Command)s.
    type Error;

    /// Applies the specified [`Event`](Aggregate::Event) when the
    /// [`State`](Aggregate::State) is empty.
    fn apply_first(event: Self::Event) -> Result<Self::State, Self::Error>;

    /// Applies the specified [`Event`](Aggregate::Event) on a pre-existing
    /// [`State`](Aggregate::State) value.
    fn apply_next(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;

    /// Handles the specified [`Command`](Aggregate::Command)when the
    /// [`State`](Aggregate::State) is empty.
    fn handle_first<'s, 'a: 's>(
        &'s self,
        id: &'a Self::Id,
        command: Self::Command,
    ) -> BoxFuture<'s, Result<Option<Vec<Self::Event>>, Self::Error>>
    where
        Self: Sized;

    /// Handles the specified [`Command`](Aggregate::Command) on a pre-existing
    /// [`State`](Aggregate::State) value.
    fn handle_next<'a, 's: 'a>(
        &'a self,
        id: &'a Self::Id,
        state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Option<Vec<Self::Event>>, Self::Error>>
    where
        Self: Sized;

    /// Translates the current [`optional::Aggregate`](Aggregate) instance into
    /// a _newtype instance_ compatible with the core
    /// [`Aggregate`](eventually_core::aggregate::Aggregate) trait.
    #[inline]
    fn as_aggregate(self) -> AsAggregate<Self>
    where
        Self: Sized,
    {
        AsAggregate::from(self)
    }
}

/// _Newtype pattern_ to ensure compatibility between
/// [`optional::Aggregate`](Aggregate) trait and the core
/// [`Aggregate`](eventually_core::aggregate::Aggregate) trait.
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
#[derive(Clone)]
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
    A::Id: Send + Sync,
    A::Command: Send + Sync,
    A::State: Send + Sync,
{
    type Id = A::Id;
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
        id: &'s Self::Id,
        state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Option<Vec<Self::Event>>, Self::Error>>
    where
        Self: Sized,
    {
        Box::pin(match state {
            None => self.0.handle_first(id, command),
            Some(state) => self.0.handle_next(id, state, command),
        })
    }
}
