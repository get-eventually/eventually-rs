#![allow(warnings, dead_code)]

use async_trait::async_trait;

use eventually::{
    aggregate::{
        optional::{AsAggregate, OptionalAggregate},
        referential::ReferentialAggregate,
        Aggregate,
    },
    command::{
        r#static::{AsHandler, StaticHandler as StaticCommandHandler},
        AggregateEvent, AggregateState, Handler as CommandHandler,
    },
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

#[derive(Debug)]
pub enum PointCommand {
    GoUp(i32),
    GoDown(i32),
    GoLeft(i32),
    GoRight(i32),
}

#[async_trait]
impl StaticCommandHandler for Point {
    type Command = PointCommand;
    type Aggregate = AsAggregate<Self>;
    type Error = std::convert::Infallible;

    async fn handle(
        state: &AggregateState<Self::Aggregate>,
        command: Self::Command,
    ) -> Result<Vec<AggregateEvent<Self::Aggregate>>, Self::Error> {
        Ok(vec![match command {
            PointCommand::GoUp(y) => PointEvent::WentUp(y),
            PointCommand::GoDown(y) => PointEvent::WentDown(y),
            PointCommand::GoLeft(x) => PointEvent::WentLeft(x),
            PointCommand::GoRight(x) => PointEvent::WentRight(x),
        }])
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

    #[test]
    fn it_handles_commands_correctly() {
        let state = None;

        let events =
            tokio_test::block_on(Point::as_handler().handle(&state, PointCommand::GoDown(5)))
                .unwrap();

        assert_eq!(
            AsAggregate::<Point>::fold(state, events.into_iter()).unwrap(),
            Some(Point(0, -5))
        );
    }
}
