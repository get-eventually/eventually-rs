//! Module exposing a [Scenario] type to test [Aggregate]s using
//! the [given-then-when canvas](https://www.agilealliance.org/glossary/gwt/).

use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use crate::aggregate::{Aggregate, Root};
use crate::event;

/// A test scenario that can be used to test an [Aggregate] and [Aggregate Root][Root]
/// using a [given-then-when canvas](https://www.agilealliance.org/glossary/gwt/) approach.
#[derive(Default, Clone, Copy)]
pub struct Scenario<T>(PhantomData<T>)
where
    T: Aggregate,
    T::Id: Clone,
    T::Event: Debug + PartialEq,
    T::Error: Debug;

impl<T> Scenario<T>
where
    T: Aggregate,
    T::Id: Clone,
    T::Event: Debug + PartialEq,
    T::Error: Debug,
{
    /// Specifies the precondition for the test [Scenario].
    ///
    /// In other words, it can be used to specify all the Domain [Event][event::Envelope]s
    /// that make up the state of the [Aggregate Root][Root].
    #[must_use]
    pub fn given(self, events: Vec<event::Envelope<T::Event>>) -> ScenarioGiven<T> {
        ScenarioGiven {
            events,
            marker: PhantomData,
        }
    }

    /// Specifies the action/mutation to execute in this [Scenario].
    ///
    /// Use this branch when testing actions/mutations that create new [Aggregate Root][Root]
    /// instances, i.e. with no prior Domain Events recorded.
    #[must_use]
    pub fn when<R, F, Err>(self, f: F) -> ScenarioWhen<T, R, F, Err>
    where
        R: From<Root<T>>,
        F: Fn() -> Result<R, Err>,
    {
        ScenarioWhen {
            mutate: f,
            marker: PhantomData,
            err_marker: PhantomData,
            root_marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct ScenarioGiven<T>
where
    T: Aggregate,
    T::Id: Clone,
    T::Event: Debug + PartialEq,
    T::Error: Debug,
{
    events: Vec<event::Envelope<T::Event>>,
    marker: PhantomData<T>,
}

impl<T> ScenarioGiven<T>
where
    T: Aggregate,
    T::Id: Clone,
    T::Event: Debug + PartialEq,
    T::Error: Debug,
{
    /// Specifies the action/mutation to execute in this [Scenario].
    ///
    /// Use this branch when testing actions/mutations that modify the state
    /// of an [Aggregate Root][Root] that already exists, by specifying its
    /// current state using [`Scenario::given`].
    ///
    /// # Panics
    ///
    /// Please note: as this method expects that an [Aggregate Root][Root] instance
    /// is available when executing the domain method, it will panic if a `Root` instance
    /// could not be obtained by rehydrating the [`Aggregate`] state through the events
    /// provided in [`Scenario::given`].
    #[must_use]
    pub fn when<R, F, Err>(self, f: F) -> ScenarioWhen<T, R, impl Fn() -> Result<R, Err>, Err>
    where
        R: From<Root<T>>,
        F: Fn(&mut R) -> Result<(), Err>,
    {
        let events = Arc::new(self.events);

        ScenarioWhen {
            marker: PhantomData,
            err_marker: PhantomData,
            root_marker: PhantomData,
            mutate: move || -> Result<R, Err> {
                let mut root: R = Root::<T>::rehydrate(events.iter().cloned())
                    .expect(
                        "no error is expected when applying domain events from a 'given' clause",
                    )
                    .expect("an aggregate root instance is expected, but none was produced")
                    .into();

                match f(&mut root) {
                    Ok(()) => Ok(root),
                    Err(err) => Err(err),
                }
            },
        }
    }
}

#[doc(hidden)]
pub struct ScenarioWhen<T, R, F, Err>
where
    T: Aggregate,
    T::Event: Debug + PartialEq,
    R: From<Root<T>>,
    F: Fn() -> Result<R, Err>,
{
    mutate: F,
    marker: PhantomData<T>,
    err_marker: PhantomData<Err>,
    root_marker: PhantomData<R>,
}

impl<T, R, F, Err> ScenarioWhen<T, R, F, Err>
where
    T: Aggregate,
    T::Event: Debug + PartialEq,
    R: From<Root<T>> + Deref<Target = Root<T>>,
    F: Fn() -> Result<R, Err>,
{
    /// Specifies that the outcome of the [Scenario] is positive, and
    /// should result in the creation of the specified Domain Events.
    #[must_use]
    pub fn then(self, result: Vec<event::Envelope<T::Event>>) -> ScenarioThen<T, R, F, Err> {
        ScenarioThen {
            mutate: self.mutate,
            expected: Ok(result),
            marker: PhantomData,
        }
    }

    /// Specified that the outcome of the [Scenario] is negative.
    ///
    /// Use this method to assert the specific Error value that the
    /// [Aggregate Root][Root] method should return.
    #[must_use]
    pub fn then_error(self, err: Err) -> ScenarioThen<T, R, F, Err> {
        ScenarioThen {
            mutate: self.mutate,
            expected: Err(err),
            marker: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct ScenarioThen<T, R, F, Err>
where
    T: Aggregate,
    T::Event: Debug + PartialEq,
    R: From<Root<T>> + Deref<Target = Root<T>>,
    F: Fn() -> Result<R, Err>,
{
    mutate: F,
    expected: Result<Vec<event::Envelope<T::Event>>, Err>,
    marker: PhantomData<R>,
}

impl<T, R, F, Err> ScenarioThen<T, R, F, Err>
where
    T: Aggregate,
    T::Event: Debug + PartialEq,
    R: From<Root<T>> + Deref<Target = Root<T>>,
    F: Fn() -> Result<R, Err>,
    Err: PartialEq + Debug,
{
    /// Runs the [Scenario] and performs the various assertion for the test.
    ///
    /// # Panics
    ///
    /// This method will panic if the assertions have not passed, making
    /// the test fail.
    pub fn assert(self) {
        let result = (self.mutate)().map(|root| root.recorded_events.clone());
        assert_eq!(self.expected, result);
    }
}
