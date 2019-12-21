//! Versioning for [Optimistic Concurrency Control] support.
//!
//! [Optimistic Concurrency Control]: https://en.wikipedia.org/wiki/Optimistic_concurrency_control

use std::ops::{Deref, DerefMut};

use async_trait::async_trait;

use crate::aggregate::{EventOf, StateOf};
use crate::command;
use crate::{Aggregate, CommandHandler};

/// Extension trait for [`CommandHandler`] to support a versioned [`Aggregate`].
///
/// For more information, check [`AsHandler`].
///
/// [`CommandHandler`]: ../command/trait.Handler.html
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`AsHandler`]: struct.AsHandler.html
pub trait CommandHandlerExt: CommandHandler + Sized {
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

impl<H> CommandHandlerExt for H where H: CommandHandler + Sized {}

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
/// #
/// # use async_trait::async_trait;
/// #
/// # use eventually::{Aggregate, CommandHandler};
/// # use eventually::command;
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
/// # #[async_trait]
/// # impl CommandHandler for MyHandler {
/// #     type Command = Command;
/// #     type Aggregate = Entity;
/// #     type Error = Infallible;
/// #
/// #     async fn handle(&self, state: &Entity, command: Self::Command) ->
/// #         command::Result<Event, Self::Error>
/// #     {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// use eventually::versioned::CommandHandlerExt;
///
/// let handler = MyHandler::new().versioned();
/// #
/// ```
///
/// [`CommandHandler`]: ../command/trait.Handler.html
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
/// [`Versioned`]: struct.Versioned.html
/// [`AsAggregate`]: struct.AsAggregate.html
pub struct AsHandler<H>(H);

#[async_trait]
impl<H> CommandHandler for AsHandler<H>
where
    H: CommandHandler + Send + Sync,
    StateOf<H::Aggregate>: Send + Sync,
    H::Command: Send,
{
    type Command = H::Command;
    // Decorated Aggregate type
    type Aggregate = AsAggregate<H::Aggregate>;
    type Error = H::Error;

    async fn handle(
        &self,
        state: &StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> command::Result<EventOf<Self::Aggregate>, Self::Error> {
        let version = state.version();

        self.0.handle(state, command).await.map(|events| {
            events
                .into_iter()
                .map(|event| Versioned::with_version(event, version + 1))
                .collect()
        })
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
/// use eventually::versioned::{AsAggregate, Versioned};
///
/// // Use by wrapping the original type in `AsAggregate::<T>`
/// let result = AsAggregate::<Entity>::apply(
///     Versioned::default(),
///     Versioned::from(Event::SomeEvent)
/// );
///
/// assert_eq!(
///     result,
///     // `Versioned::from` assigns the default version to the event, which is 0.
///     // So, the state will take the same version.
///     Ok(Versioned::with_version(Entity::default(), 0)),
/// );
///
/// // If applying an event with version "1", the state will have version "1" too.
/// assert_eq!(
///     AsAggregate::<Entity>::apply(result.unwrap(), Versioned::with_version(Event::SomeEvent, 1)),
///     Ok(Versioned::with_version(Entity::default(), 1)),
/// );
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
    type Event = Versioned<A::Event>;
    type Error = A::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        let version = event.version();

        A::apply(state.data, event.take()).map(|state| Versioned::with_version(state, version))
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
