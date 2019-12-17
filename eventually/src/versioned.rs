//! Versioning for [Optimistic Concurrency Control] support.
//!
//! [Optimistic Concurrency Control]: https://en.wikipedia.org/wiki/Optimistic_concurrency_control

use std::ops::{Deref, DerefMut};

use crate::{aggregate::StateOf, Aggregate, CommandHandler};

/// Extension trait for [`CommandHandler`] to support a versioned [`Aggregate`].
///
/// For more information, check [`AsHandler`].
///
/// [`CommandHandler`]: ../command/trait.Handler.html
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`AsHandler`]: struct.AsHandler.html
pub trait HandlerExt: CommandHandler + Sized {
    /// Returns a decorated version of the [`CommandHandler`],
    /// in the form of [`AsHandler`] handler implementation.
    ///
    /// Check [`AsHandler`] documentation for more information.
    ///
    /// [`CommandHandler`]: ../command/trait.Handler.html
    /// [`AsHandler`]: struct.AsHandler.html
    fn versioned(self) -> AsHandler<Self> {
        AsHandler(self)
    }
}

impl<H> HandlerExt for H where H: CommandHandler + Sized {}

/// A [`CommandHandler`] decorator to support versioned [`Aggregate`].
///
/// This decorator uses the [`AsAggregate`] decorator as its Aggregate.
///
/// For more information, check [`AsAggregate`] documentation.
///
/// # Examples
///
/// ```
/// # use std::convert::Infallible;
/// # use futures::future::Ready;
/// # use eventually::{Aggregate, CommandHandler};
/// #
/// # enum Event {}
/// # enum Command {}
/// #
/// # struct Entity {}
/// #
/// # impl Aggregate for Entity {
/// #     type State = Self;
/// #     type Event = Event;
/// #     type Error = Infallible;
/// #
/// #     fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// # struct MyHandler;
/// # impl MyHandler {
/// #     fn new() -> Self {
/// #         MyHandler
/// #     }
/// # }
/// #
/// # impl CommandHandler for MyHandler {
/// #     type Command = Command;
/// #     type Aggregate = Entity;
/// #     type Error = Infallible;
/// #     type Result = Ready<Result<Vec<Event>, Self::Error>>;
/// #
/// #     fn handle(&self, state: &Entity, command: Self::Command) -> Self::Result {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// # fn main() {
/// use eventually::versioned::HandlerExt;
///
/// let handler = MyHandler::new().versioned();
/// # }
/// ```
///
/// [`CommandHandler`]: ../command/trait.Handler.html
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`Versioned`]: struct.Versioned.html
/// [`AsAggregate`]: struct.AsAggregate.html
pub struct AsHandler<H>(H);

impl<H: CommandHandler> CommandHandler for AsHandler<H> {
    type Command = H::Command;
    // Decorated Aggregate type
    type Aggregate = AsAggregate<H::Aggregate>;
    type Error = H::Error;

    // NOTE: it'd be nicer if we could also map from the decorated Handler result
    // to versioned events.
    //
    // That way, many events produced by a single command would yield the same version.
    type Result = H::Result;

    fn handle(&self, state: &StateOf<Self::Aggregate>, command: Self::Command) -> Self::Result {
        self.0.handle(state, command)
    }
}

/// An [`Aggregate`] decorator that supports versioning.
///
/// Versioned [`State`] is possible through the [`Versioned`] type wrapper.
///
/// # Examples
///
/// ```
/// use std::convert::Infallible;
/// use eventually::Aggregate;
///
/// enum Event {
///     SomeEvent
/// }
///
/// #[derive(Debug, PartialEq)]
/// struct Entity {}
///
/// impl Default for Entity {
///     fn default() -> Self {
///         Entity {}
///     }
/// }
///
/// impl Aggregate for Entity {
///     type State = Self;
///     type Event = Event;
///     type Error = Infallible;
///
///     fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
///         // Simple dumb implementation, you'll probably want something
///         // more interesting in your code ;-)
///         Ok(state)
///     }
/// }
///
/// fn main() {
///     use eventually::versioned::{AsAggregate, Versioned};
///
///     // Use by wrapping the original type in `AsAggregate::<T>`
///     let result = AsAggregate::<Entity>::apply(Versioned::default(), Event::SomeEvent);
///
///     assert_eq!(
///         result,
///         // Wraps the Entity instance with version "1"
///         Ok(Versioned::new(Entity::default(), 1)),
///     );
///
///     // If applied on a versioned state again, the version will increase
///     // from "1" to "2"
///     assert_eq!(
///         AsAggregate::<Entity>::apply(result.unwrap(), Event::SomeEvent),
///         Ok(Versioned::new(Entity::default(), 2)),
///     );
/// }
/// ```
///
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`State`]: ../aggregate/trait.Aggregate.html#associatedtype.State
/// [`Versioned`]: struct.Versioned.html
pub struct AsAggregate<A>(std::marker::PhantomData<A>);

impl<A> Aggregate for AsAggregate<A>
where
    A: Aggregate,
{
    type State = Versioned<A::State>;
    type Event = A::Event;
    type Error = A::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        let version = state.version();

        A::apply(state.data, event).map(|state| Versioned::with_version(state, version + 1))
    }
}

/// Wrapper to embed version information for un-versioned data types.
#[derive(Debug, Clone, PartialEq)]
pub struct Versioned<T> {
    data: T,
    version: u64,
}

impl<T> Versioned<T> {
    /// Wraps data with the specified version information.
    pub fn with_version(data: T, version: u64) -> Self {
        Versioned { data, version }
    }

    /// Returns version information.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Extracts the wrapped data from the instance.
    pub fn take(self) -> T {
        self.data
    }
}

impl<T> Deref for Versioned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Versioned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> From<T> for Versioned<T> {
    fn from(data: T) -> Self {
        Versioned::with_version(data, 0)
    }
}

impl<T> Default for Versioned<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::from(T::default())
    }
}
