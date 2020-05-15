//! Contains support for _Optimistic Concurrency Control_ using
//! _versioning attributes_.
//!
//! Check out [`AggregateExt`] for more information.
//!
//! [`AggregateExt`]: trait.AggregateExt.html

use std::ops::{Deref, DerefMut};

use eventually_core::aggregate::{Aggregate, Identifiable};

use futures::future::BoxFuture;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Extension trait to add _Optimistic Concurrency Control_ support
/// over an [`Aggregate`] using _versioning_.
///
/// ## Usage
///
/// Call [`versioned`] over an [`Aggregate`] instance to add versioning support.
///
/// ```text
/// use eventually_util::versioned::AggregateExt;
///
/// // Assuming `SomeAggregateExample` is an Aggregate
/// let versioned_aggregate = SomeAggregateExample.versioned();
/// ```
///
/// [`Aggregate`]: ../../eventually_core/aggregate/Aggregate.html
/// [`versioned`]: trait.AggregateExt.html#method.versioned
pub trait AggregateExt: Aggregate + Sized {
    /// Returns a _versioned_ flavour of an [`Aggregate`].
    ///
    /// [`Aggregate`]: ../../eventually_core/aggregate/Aggregate.html
    #[inline]
    fn versioned(self) -> AsAggregate<Self> {
        AsAggregate(self)
    }
}

impl<T> AggregateExt for T where T: Aggregate + Sized {}

/// _Newtype_ extension for [`Aggregate`] types to add support
/// for _Optimistic Concurrency Control_ using _versioning_ for [`Event`]s
/// and Aggregate [`State`].
///
/// Check out [`AggregateExt`] for more information.
///
/// [`Aggregate`]: ../../eventually_core/aggregate/trait.Aggregate.html
/// [`Event`]: ../../eventually_core/aggregate/trait.Aggregate.html#associatedtype.Event
/// [`State`]: ../../eventually_core/aggregate/trait.Aggregate.html#associatedtype.State
/// [`AggregateExt`]: trait.AggregateExt.html
#[derive(Debug, Clone)]
pub struct AsAggregate<T>(T);

impl<T> Aggregate for AsAggregate<T>
where
    T: Aggregate + Send + Sync,
    T::State: Send + Sync,
    T::Command: Send + Sync,
{
    type State = Versioned<T::State>;
    type Event = Versioned<T::Event>;
    type Command = T::Command;
    type Error = T::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        let version = event.version();
        let event = event.take();
        let state = state.data;

        T::apply(state, event).map(|state| Versioned::new(state, version))
    }

    fn handle<'a, 's: 'a>(
        &'a self,
        state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized,
    {
        let version = state.version();

        Box::pin(async move {
            self.0.handle(state, command).await.map(|events| {
                events
                    .into_iter()
                    .map(|event| Versioned::new(event, version + 1))
                    .collect()
            })
        })
    }
}

/// Wrapper to embed version information for un-versioned data types.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Versioned<T> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    data: T,
    version: u32,
}

impl<T> Versioned<T> {
    /// Wraps data with the specified version information.
    #[inline]
    pub fn new(data: T, version: u32) -> Self {
        Versioned { data, version }
    }

    /// Returns version information.
    #[inline]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Extracts the wrapped data from the instance.
    #[inline]
    pub fn take(self) -> T {
        self.data
    }
}

impl<T> Deref for Versioned<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Versioned<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> From<T> for Versioned<T> {
    #[inline]
    fn from(data: T) -> Self {
        Versioned::new(data, 0)
    }
}

impl<T> Default for Versioned<T>
where
    T: Default,
{
    #[inline]
    fn default() -> Self {
        Self::from(T::default())
    }
}

impl<T> Identifiable for Versioned<T>
where
    T: Identifiable,
{
    type Id = T::Id;

    #[inline]
    fn id(&self) -> Self::Id {
        self.data.id()
    }
}
