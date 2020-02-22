//! Contains [`Aggregate`] variant where aggregate root and [`State`]
//! are implemented by the same object.
//!
//! [`Aggregate`]: ../trait.Aggregate.html
//! [`State`]: ../trait.Aggregate.html#associatedType.State

use eventually_core::aggregate;

/// An [`Aggregate`] representation where the [`State`]
/// is the same as the aggregate root.
///
/// # Examples
///
/// ```
/// use eventually::aggregate::referential::Aggregate;
///
/// struct MyState {
///     // Some fields
/// }
///
/// enum MyEvent {
///     Happened,
/// }
///
/// impl Aggregate for MyState {
///     type Event = MyEvent;
///     type Error = std::convert::Infallible;
///
///     fn apply(self, event: Self::Event) -> Result<Self, Self::Error> {
///         unimplemented!()
///     }
/// }
/// ```
///
/// [`Aggregate`]: ../trait.Aggregate.html
/// [`State`]: ../trait.Aggregate.html#associatedType.State
pub trait Aggregate: Sized {
    /// Domain events that express Aggregate's mutations.
    ///
    /// Check out [`Event`] documentation for more information.
    ///
    /// [`Event`]: ../trait.Aggregate.html#associatedType.Event
    type Event;

    /// Error returned by [`apply`] when handling an incorrect [`Event`].
    ///
    /// Check out [`Error`] documentation for more information.
    ///
    /// [`apply`]: trait.Aggregate.html#method.apply
    /// [`Event`]: trait.Aggregate.html#associatedType.Event
    /// [`Error`]: ../trait.Aggregate.html#associatedType.Error
    type Error;

    /// Applies an [`Event`] to the current state of the Aggregate
    /// (`Self` in this instance), returning either the next state
    /// or an [`Error`].
    ///
    /// [`Event`]: trait.Aggregate.html#associatedType.Event
    /// [`Error`]: trait.Aggregate.html#associatedType.Error
    fn apply(self, event: Self::Event) -> Result<Self, Self::Error>;
}

/// Adapter for [`Aggregate`] types to the foundational [`eventually::Aggregate`] trait.
///
/// [`Aggregate`]: trait.Aggregate.html
/// [`eventually::Aggregate`]: ../trait.Aggregate.html
pub struct AsAggregate<T>(std::marker::PhantomData<T>);

impl<T> aggregate::Aggregate for AsAggregate<T>
where
    T: Aggregate,
{
    type State = T;
    type Event = T::Event;
    type Error = T::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        T::apply(state, event)
    }
}
