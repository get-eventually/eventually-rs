use std::ops::{Deref, DerefMut};

use super::Aggregate;

pub trait Versioned {
    fn version(&self) -> u64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct State<T> {
    pub data: T,
    pub version: u64,
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> From<T> for State<T> {
    fn from(data: T) -> Self {
        State { data, version: 0 }
    }
}

impl<T> Default for State<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::from(T::default())
    }
}

impl<T> Versioned for State<T> {
    fn version(&self) -> u64 {
        self.version
    }
}

pub struct AsAggregate<A>(std::marker::PhantomData<A>);

impl<A> Aggregate for AsAggregate<A>
where
    A: Aggregate,
{
    type State = State<A::State>;
    type Event = A::Event;
    type Error = A::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        let version = state.version;

        A::apply(state.data, event).map(|state| State {
            data: state,
            version: version + 1,
        })
    }
}
