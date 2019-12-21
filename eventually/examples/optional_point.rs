use async_trait::async_trait;

use eventually::aggregate::{EventOf, ReferentialAggregate, StateOf};
use eventually::command;
use eventually::command::r#static::StaticHandler as StaticCommandHandler;
use eventually::optional::{Aggregate, AsAggregate};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point(i32, i32);

impl Point {
    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }
}

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

impl Aggregate for Point {
    type State = Self;
    type Event = PointEvent;
    type Error = std::convert::Infallible;

    fn apply_first(event: Self::Event) -> Result<Self::State, Self::Error> {
        Point(0, 0).apply(event)
    }

    fn apply_next(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
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
        _state: &StateOf<Self::Aggregate>,
        command: Self::Command,
    ) -> command::Result<EventOf<Self::Aggregate>, Self::Error> {
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

    use eventually::{
        versioned::{AsAggregate as VersionedAggregate, Versioned},
        Aggregate, AggregateExt, CommandHandler,
    };

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

    #[test]
    fn it_folds_data_by_using_versioned_aggregate_trait() {
        let state = VersionedAggregate::<AsAggregate<Point>>::fold(
            Versioned::default(),
            vec![
                PointEvent::WentUp(10),
                PointEvent::WentRight(10),
                PointEvent::WentDown(5),
            ]
            .into_iter(),
        )
        .unwrap();

        assert_eq!(state.version(), 3);

        // Testing that Deref is working appropriately
        assert_eq!(state.as_ref().unwrap().x(), 10);
        assert_eq!(state.as_ref().unwrap().y(), 5);

        assert_eq!(state.take(), Some(Point(10, 5)));
    }
}
