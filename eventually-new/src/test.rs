use async_trait::async_trait;

use chrono::NaiveDateTime;

use crate::aggregate::Aggregate;
use crate::{Event, Events};

#[derive(Debug, Clone)]
pub struct OrderState {
    finalized: bool,
    updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Copy)]
pub struct Order;

#[derive(Debug)]
pub enum OrderCommand {
    Create { id: String, now: NaiveDateTime },
    Finalize { id: String, now: NaiveDateTime },
}

impl AsRef<String> for OrderCommand {
    fn as_ref(&self) -> &String {
        match self {
            OrderCommand::Create { id, .. } => id,
            OrderCommand::Finalize { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderEvent {
    Created { at: NaiveDateTime },
    Finalized { at: NaiveDateTime },
}

// #[async_trait]
// impl NewAggregate for Order {
//     type Command = OrderCreate;
//     type NewError = std::convert::Infallible;

//     async fn new(
//         &mut self,
//         command: <Self as NewAggregate>::Command,
//     ) -> Result<Vec<Event<<Self as Aggregate>::DomainEvent>>, Self::NewError> {
//         Ok(vec![Event::new(OrderEvent::Created { at: command.now })])
//     }

//     fn apply_first(event: Event<Self::DomainEvent>) -> Result<Self::State, Self::ApplyError> {
//         Ok(Some(match event.into_inner() {
//             OrderEvent::Created { at } => OrderState {
//                 finalized: false,
//                 updated_at: at,
//             },

//             OrderEvent::Finalized { at } => OrderState {
//                 finalized: true,
//                 updated_at: at,
//             },
//         }))
//     }
// }

#[async_trait]
impl Aggregate for Order {
    type Id = String;
    type Command = OrderCommand;
    type State = Option<OrderState>;
    type DomainEvent = OrderEvent;
    type HandleError = std::convert::Infallible;
    type ApplyError = std::convert::Infallible;

    async fn handle(
        &mut self,
        _: &Self::State,
        command: Self::Command,
    ) -> Result<Option<Events<Self::DomainEvent>>, Self::HandleError> {
        Ok(Some(vec![Event::from(match command {
            OrderCommand::Create { now, .. } => OrderEvent::Created { at: now },
            OrderCommand::Finalize { now, .. } => OrderEvent::Finalized { at: now },
        })]))
    }

    fn apply(
        _state: Self::State,
        event: Event<Self::DomainEvent>,
    ) -> Result<Self::State, Self::ApplyError> {
        Ok(Some(match event.into_inner() {
            OrderEvent::Created { at } => OrderState {
                finalized: false,
                updated_at: at,
            },

            OrderEvent::Finalized { at } => OrderState {
                finalized: true,
                updated_at: at,
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;

    use crate::scenario::AggregateRootScenario;

    #[tokio::test]
    async fn it_works() {
        let id = "test-order";
        let now = Utc::now().naive_utc();

        AggregateRootScenario::with(id.to_owned(), Order)
            .when(OrderCommand::Create {
                id: id.to_owned(),
                now,
            })
            .then(Some(vec![OrderEvent::Created { at: now }.into()]))
            .await;
    }
}
