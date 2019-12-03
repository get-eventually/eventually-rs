#![allow(warnings, dead_code)]

use eventually::aggregate::{
    referential::{AsAggregate, ReferentialAggregate},
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

impl Default for Point {
    fn default() -> Self {
        Point(0, 0)
    }
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

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_folds_lots_of_events_together() {
        let result = Point::default()
            .fold(
                vec![
                    PointEvent::WentUp(10),
                    PointEvent::WentRight(10),
                    PointEvent::WentDown(5),
                ]
                .into_iter(),
            )
            .unwrap();

        assert_eq!(result, Point(10, 5));
    }

    #[test]
    fn it_folds_data_by_using_aggregate_trait() {
        assert_eq!(
            AsAggregate::<Point>::fold(
                Point::default(),
                vec![
                    PointEvent::WentUp(10),
                    PointEvent::WentRight(10),
                    PointEvent::WentDown(5),
                ]
                .into_iter(),
            ),
            Point::default().fold(
                vec![
                    PointEvent::WentUp(10),
                    PointEvent::WentRight(10),
                    PointEvent::WentDown(5),
                ]
                .into_iter(),
            )
        );
    }
}
