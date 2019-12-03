use crate::aggregate::Aggregate;

pub trait OptionalAggregate {
    type State;
    type Event;
    type Error;

    fn initial(event: Self::Event) -> Result<Self::State, Self::Error>;

    fn apply_next(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;
}

pub struct AsAggregate<T>(std::marker::PhantomData<T>);

impl<T: OptionalAggregate> Aggregate for AsAggregate<T> {
    type State = Option<T::State>;
    type Event = T::Event;
    type Error = T::Error;

    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        Ok(Some(match state {
            None => T::initial(event)?,
            Some(state) => T::apply_next(state, event)?,
        }))
    }
}
