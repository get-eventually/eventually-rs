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

#[cfg(test)]
mod tests {
    use super::{AggregateExt, Versioned};

    use eventually_core::aggregate::{Aggregate, AggregateRoot, Identifiable};

    use futures::future::BoxFuture;

    use tokio_test::block_on;

    #[test]
    fn aggregate_versioning_extension_works() {
        let aggregate = PointAggregate.versioned();
        let state = Versioned::from(Point { x: 0f32, y: 0f32 });
        let mut root = AggregateRoot::new(aggregate, state);

        block_on(async {
            root.handle(PointCommand::Rotate {
                anchor: (0f32, 0f32),
                degrees: 90f32,
            })
            .await
            .expect("should be infallible")
            .handle(PointCommand::Rotate {
                anchor: (0f32, 0f32),
                degrees: 90f32,
            })
            .await
            .expect("should be infallible")
            .handle(PointCommand::Rotate {
                anchor: (0f32, 0f32),
                degrees: 90f32,
            })
            .await
            .expect("should be infallible")
            .handle(PointCommand::Rotate {
                anchor: (0f32, 0f32),
                degrees: 90f32,
            })
            .await
            .expect("should be infallible")
        });

        assert_eq!(root.state(), &Versioned::new(Point { x: 0f32, y: 0f32 }, 4))
    }

    #[derive(Debug, PartialEq, Clone, Copy, Default)]
    struct Point {
        x: f32,
        y: f32,
    }

    // We don't care about identity for this example.
    impl Identifiable for Point {
        type Id = ();

        fn id(&self) -> Self::Id {
            ()
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct PointUpdated {
        x: f32,
        y: f32,
    }

    #[derive(Debug)]
    enum PointCommand {
        Rotate { anchor: (f32, f32), degrees: f32 },
    }

    #[derive(Debug, Clone, Copy)]
    struct PointAggregate;
    impl Aggregate for PointAggregate {
        type State = Point;
        type Event = PointUpdated;
        type Command = PointCommand;
        type Error = std::convert::Infallible;

        fn apply(mut state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
            state.x = event.x;
            state.y = event.y;

            Ok(state)
        }

        fn handle<'a, 's: 'a>(
            &'a self,
            state: &'s Self::State,
            command: Self::Command,
        ) -> BoxFuture<'a, Result<Vec<Self::Event>, Self::Error>>
        where
            Self: Sized,
        {
            Box::pin(futures::future::ok(match command {
                PointCommand::Rotate { anchor, degrees } => {
                    let angle = degrees * std::f32::consts::PI;
                    let (center_x, center_y) = anchor;

                    let delta_x = state.x - center_x;
                    let delta_y = state.y - center_y;

                    let rotated_x = (angle.cos() * delta_x - angle.sin() * delta_y) + center_x;
                    let rotated_y = (angle.sin() * delta_x - angle.cos() * delta_y) + center_y;

                    vec![PointUpdated {
                        x: rotated_x,
                        y: rotated_y,
                    }]
                }
            }))
        }
    }
}
