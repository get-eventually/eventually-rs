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

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum OrderError {
    #[error("order has already been created")]
    AlreadyCreated,

    #[error("order not been created yet")]
    NotCreatedYet,

    #[error("order has already been finalized")]
    AlreadyFinalized,
}

#[async_trait]
impl Aggregate for Order {
    type Id = String;
    type State = Option<OrderState>;
    type DomainEvent = OrderEvent;
    type Command = OrderCommand;
    type ApplyError = std::convert::Infallible;
    type HandleError = OrderError;

    async fn handle(
        &mut self,
        state: &<Self as Aggregate>::State,
        command: Self::Command,
    ) -> Result<Events<<Self as Aggregate>::DomainEvent>, Self::HandleError> {
        match command {
            OrderCommand::Create { now, .. } => {
                if state.is_none() {
                    Ok(vec![Event::from(OrderEvent::Created { at: now })])
                } else {
                    Err(OrderError::AlreadyCreated)
                }
            }

            OrderCommand::Finalize { now, .. } => match state {
                None => Err(OrderError::NotCreatedYet),
                Some(state) => {
                    if state.finalized {
                        Err(OrderError::AlreadyFinalized)
                    } else {
                        Ok(vec![Event::from(OrderEvent::Finalized { at: now })])
                    }
                }
            },
        }
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
    async fn order_is_created() {
        let id = "test-order".to_owned();
        let now = Utc::now().naive_utc();

        AggregateRootScenario::with(id.clone(), Order)
            .when(OrderCommand::Create { id, now })
            .then(vec![OrderEvent::Created { at: now }.into()])
            .await;
    }

    #[tokio::test]
    async fn order_is_finalized() {
        let id = "test-order".to_owned();
        let now = Utc::now().naive_utc();
        let created_at = now - chrono::Duration::days(1);

        AggregateRootScenario::with(id.clone(), Order)
            .given(vec![OrderEvent::Created { at: created_at }.into()])
            .when(OrderCommand::Finalize { id, now })
            .then(vec![OrderEvent::Finalized { at: now }.into()])
            .await;
    }

    #[tokio::test]
    async fn order_cannot_be_created_twice() {
        let id = "test-order".to_owned();
        let now = Utc::now().naive_utc();
        let created_at = now - chrono::Duration::days(1);

        AggregateRootScenario::with(id.clone(), Order)
            .given(vec![OrderEvent::Created { at: created_at }.into()])
            .when(OrderCommand::Create { id, now })
            .then_error(OrderError::AlreadyCreated)
            .await;
    }

    #[tokio::test]
    async fn order_cannot_be_finalized_twice() {
        let id = "test-order".to_owned();
        let now = Utc::now().naive_utc();
        let created_at = now - chrono::Duration::days(1);

        AggregateRootScenario::with(id.clone(), Order)
            .given(vec![
                OrderEvent::Created { at: created_at }.into(),
                OrderEvent::Finalized { at: now }.into(),
            ])
            .when(OrderCommand::Finalize { id, now })
            .then_error(OrderError::AlreadyFinalized)
            .await;
    }

    #[tokio::test]
    async fn order_cannot_be_finalized_if_not_created() {
        let id = "test-order".to_owned();
        let now = Utc::now().naive_utc();

        AggregateRootScenario::with(id.clone(), Order)
            .when(OrderCommand::Finalize { id, now })
            .then_error(OrderError::NotCreatedYet)
            .await;
    }
}
