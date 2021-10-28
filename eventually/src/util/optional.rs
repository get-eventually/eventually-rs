//! Contains a different flavour of the [`Aggregate`] trait,
//! while still maintaining compatibility through [`IntoAggregate`] type.
//!
//! Check out [`optional::Aggregate`](Aggregate) for more information.

use async_trait::async_trait;

/// An [`Option`]-flavoured, [`Aggregate`]-compatible trait
/// to model Aggregates having an optional [`State`](Aggregate::State).
///
/// Use [`as_aggregate`](Aggregate::as_aggregate) to get an
/// [`Aggregate`]-compatible instance of this trait.
#[async_trait]
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
    async fn handle_first(
        &self,
        id: &Self::Id,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error>;

    /// Handles the specified [`Command`](Aggregate::Command) on a pre-existing
    /// [`State`](Aggregate::State) value.
    async fn handle_next(
        &self,
        id: &Self::Id,
        state: &Self::State,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error>;

    /// Translates the current [`optional::Aggregate`](Aggregate) instance into
    /// a _newtype instance_ compatible with the core
    /// [`Aggregate`](crate::aggregate::Aggregate) trait.
    #[inline]
    fn into_aggregate(self) -> IntoAggregate<Self>
    where
        Self: Sized,
    {
        IntoAggregate::from(self)
    }
}

/// _Newtype pattern_ to ensure compatibility between
/// [`optional::Aggregate`](Aggregate) trait and the core
/// [`Aggregate`](crate::aggregate::Aggregate) trait.
///
/// ## Usage
///
/// 1. Use `From<Aggregate>` trait implementation:
///     ```text
///     use eventually_util::optional::IntoAggregate;
///
///     let aggregate = IntoAggregate::from(MyOptionalAggregate);
///     ```
/// 2. Use the [`Aggregate::as_aggregate`] method:
///     ```text
///     let aggregate = MyOptionalAggregate.as_aggregate();
///     ```
#[derive(Clone)]
pub struct IntoAggregate<A>(A);

impl<A> From<A> for IntoAggregate<A> {
    #[inline]
    fn from(value: A) -> Self {
        IntoAggregate(value)
    }
}

#[async_trait]
impl<A> crate::aggregate::Aggregate for IntoAggregate<A>
where
    A: Aggregate + Send + Sync,
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

    async fn handle(
        &self,
        id: &Self::Id,
        state: &Self::State,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match state {
            None => self.0.handle_first(id, command).await,
            Some(state) => self.0.handle_next(id, state, command).await,
        }
    }
}
