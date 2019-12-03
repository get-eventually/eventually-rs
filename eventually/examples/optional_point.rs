#![allow(warnings, dead_code)]

use eventually::aggregate::{
    optional::{AsAggregate, OptionalAggregate},
    referential::ReferentialAggregate,
    Aggregate,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Point(i32, i32);

#[derive(Debug)]
pub enum PointEvent {
    WentUp(i32),
    WentDown(i32),
    WentLeft(i32),
    WentRight(i32),
}

impl ReferentialAggregate for Point {
    type Event = PointEvent;
    type Error = std::convert::Infallible;

    fn apply(self, event: Self::Event) -> Result<Self, Self::Error> {
        Ok(match event {
            PointEvent::WentUp(quantity) => Point(self.0, self.1 + quantity),
            PointEvent::WentDown(quantity) => Point(self.0, self.1 - quantity),
            PointEvent::WentLeft(quantity) => Point(self.0 - quantity, self.1),
            PointEvent::WentRight(quantity) => Point(self.0 + quantity, self.1),
        })
    }
}

impl OptionalAggregate for Point {
    type State = Self;
    type Event = PointEvent;
    type Error = std::convert::Infallible;

    fn initial(event: Self::Event) -> Result<Self::State, Self::Error> {
        Point(0, 0).apply(event)
    }

    fn apply_next(mut state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        state.apply(event)
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_applies_an_event_correctly() {
        assert_eq!(
            AsAggregate::<Point>::apply(None, PointEvent::WentUp(10)),
            Ok(Some(Point(0, 10))),
        );
    }

    #[test]
    fn it_folds_data_by_using_aggregate_trait() {
        assert_eq!(
            AsAggregate::<Point>::fold(
                None,
                vec![
                    PointEvent::WentUp(10),
                    PointEvent::WentRight(10),
                    PointEvent::WentDown(5),
                ]
                .into_iter()
            ),
            Ok(Some(Point(10, 5)))
        );
    }
}
