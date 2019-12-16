use crate::aggregate::Aggregate;

pub trait ReferentialAggregate: Sized {
    type Event;
    type Error;

    fn apply(self, event: Self::Event) -> Result<Self, Self::Error>;
}

pub struct AsAggregate<T>(std::marker::PhantomData<T>);

impl<T: ReferentialAggregate> Aggregate for AsAggregate<T> {
    type State = T;
    type Event = T::Event;
    type Error = T::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        T::apply(state, event)
    }
}
