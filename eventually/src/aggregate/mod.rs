//! Foundation traits for creating Domain abstractions
//! using [the `Aggregate` pattern](https://martinfowler.com/bliki/DDD_Aggregate.html).
//!
//! For ease of development, the following modules contain different
//! traits for modeling [`Aggregate`]-compatible structures:
//!
//! * [`referential`], containing an [`Aggregate`]-compatible trait where
//! the trait is implemented for `self`, thus making [`State`] as `Self`.
//!
//! [`Aggregate`]: trait.Aggregate.html
//! [`State`]: trait.Aggregate.html#associatedtype.State
//! [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
//!
//! [`referential`]: referential/index.html

pub mod referential;

pub use referential::ReferentialAggregate;

use futures::{future::BoxFuture, Stream, StreamExt};

/// Alias for the [`State`] type of an [`Aggregate`].
///
/// [`State`]: trait.Aggregate.html#associatedtype.State
/// [`Aggregate`]: trait.Aggregate.html
pub type StateOf<A: Aggregate> = A::State;

/// Alias for the [`Event`] type of an [`Aggregate`].
///
/// [`Event`]: trait.Aggregate.html#associatedtype.Event
/// [`Aggregate`]: trait.Aggregate.html
pub type EventOf<A: Aggregate> = A::Event;

/// Alias for the [`Error`] type of an [`Aggregate`].
///
/// [`Error`]: trait.Aggregate.html#associatedtype.Error
/// [`Aggregate`]: trait.Aggregate.html
pub type ErrorOf<A: Aggregate> = A::Error;

/// An Aggregate is an entity which [`State`] is composed of one or more
/// _value-objects_, _entities_ or nested _aggregates_.
///
/// State mutations are expressed through clear _Domain Events_ which, if
/// applied in the same order as they happened _chronologically_, will yield
/// the same [`State`] value.
///
/// [`State`]: trait.Aggregate.html#associatedtype.State
pub trait Aggregate {
    /// State of the Aggregate.
    ///
    /// Usually this associate type is either `Self`, `Option<Self>` or
    /// [`Option<T>`], depending on whether the Aggregate state is defined
    /// in a separate data structure or using the same structure that
    /// implements this trait.
    ///
    /// [`Option<T>`]: https://doc.rust-lang.org/std/option/enum.Option.html
    type State;

    /// Domain events that express mutations of the Aggregate's [`State`].
    ///
    /// Usually, this type is an `enum` containing all possible
    /// _Domain Events_ possible for this [`Aggregate`].
    ///
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Aggregate`]: trait.Aggregate.html
    type Event;

    /// Error type returned in [`apply`] when mutating the Aggregate State
    /// to the next version fails.
    ///
    /// Usually, this error is a validation error type raised when the
    /// domain event that is being applied is invalid, based on the current [`State`].
    ///
    /// Consider using [`std::convert::Infallible`] (or `!` type if using nightly)
    /// if the [`apply`] method doesn't fail.
    ///
    /// [`apply`]: trait.Aggregate.html#tymethod.apply
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`std::convert::Infallible`]: https://doc.rust-lang.org/std/convert/enum.Infallible.html
    type Error;

    /// Applies the [`Event`] to the current [`State`],
    /// returning either the next [`State`] or an [`Error`].
    ///
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    /// [`Error`]: trait.Aggregate.html#associatedtype.Error
    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;
}

/// Extension trait for [`Aggregate`] containing combinator functions.
///
/// [`Aggregate`]: trait.Aggregate.html
pub trait AggregateExt: Aggregate {
    /// Applies a _synchronous_ stream of [`Event`]s to the current [`State`],
    /// returning the updated state or an error, if any such happened.
    ///
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    #[inline]
    fn fold(
        state: Self::State,
        events: impl Iterator<Item = Self::Event>,
    ) -> Result<Self::State, Self::Error> {
        events.fold(Ok(state), |previous, event| {
            previous.and_then(|state| Self::apply(state, event))
        })
    }

    /// Applies an _asynchronous_ [`Stream`] of [`Event`]s to the current
    /// [`State`], returning the updated state or an error, if any such happened.
    ///
    /// [`Stream`]: ../../futures/stream/trait.Stream.html
    /// [`Event`]: trait.Aggregate.html#associatedtype.Event
    /// [`State`]: trait.Aggregate.html#associatedtype.State
    #[inline]
    fn async_fold<'a>(
        state: Self::State,
        events: impl Stream<Item = Self::Event> + Send + 'a,
    ) -> BoxFuture<'a, Result<Self::State, Self::Error>>
    where
        Self::State: Send + 'a,
        Self::Event: Send + 'a,
        Self::Error: Send + 'a,
    {
        Box::pin(events.fold(Ok(state), |previous, event| {
            async move { previous.and_then(|state| Self::apply(state, event)) }
        }))
    }
}

impl<T> AggregateExt for T where T: Aggregate {}
