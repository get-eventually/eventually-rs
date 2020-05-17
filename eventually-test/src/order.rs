use std::fmt::{Display, Formatter, Result as FmtResult};

use chrono::{DateTime, Utc};

use futures::{future, future::BoxFuture};

use serde::{Deserialize, Serialize};

use eventually::optional::Aggregate;
use eventually::Identifiable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub item_sku: String,
    pub quantity: u8,
    pub price: f32,
}

pub struct OrderItems(Vec<OrderItem>);

impl From<Vec<OrderItem>> for OrderItems {
    fn from(value: Vec<OrderItem>) -> Self {
        OrderItems(value)
    }
}

impl From<OrderItems> for Vec<OrderItem> {
    fn from(value: OrderItems) -> Self {
        value.0
    }
}

impl OrderItems {
    fn insert_or_merge(self, item: OrderItem) -> Self {
        let mut list: Vec<OrderItem> = self.into();

        list.iter_mut()
            .find(|it| item.item_sku == it.item_sku)
            .map(|it| it.quantity += item.quantity)
            .or_else(|| Some(list.push(item)));

        OrderItems::from(list)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum OrderState {
    Editable {
        id: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        items: Vec<OrderItem>,
    },

    Complete {
        id: String,
        created_at: DateTime<Utc>,
        items: Vec<OrderItem>,
        completed_at: DateTime<Utc>,
    },

    Cancelled {
        id: String,
        created_at: DateTime<Utc>,
        items: Vec<OrderItem>,
        cancelled_at: DateTime<Utc>,
    },
}

impl Identifiable for OrderState {
    type Id = String;

    fn id(&self) -> Self::Id {
        match self {
            OrderState::Editable { id, .. } => id.clone(),
            OrderState::Complete { id, .. } => id.clone(),
            OrderState::Cancelled { id, .. } => id.clone(),
        }
    }
}

#[derive(Debug)]
pub enum OrderCommand {
    Create { id: String },
    AddItem { item: OrderItem },
    Complete,
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum OrderEvent {
    Created { id: String, at: DateTime<Utc> },
    ItemAdded { item: OrderItem, at: DateTime<Utc> },
    Completed { at: DateTime<Utc> },
    Cancelled { at: DateTime<Utc> },
}

impl OrderEvent {
    pub fn happened_at(&self) -> &DateTime<Utc> {
        match self {
            OrderEvent::Created { at, .. } => at,
            OrderEvent::ItemAdded { at, .. } => at,
            OrderEvent::Completed { at, .. } => at,
            OrderEvent::Cancelled { at, .. } => at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderError {
    AlreadyCreated,
    NotYetCreated,
    NotEditable,
    AlreadyCompleted,
    AlreadyCancelled,
}

impl Display for OrderError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match self {
            OrderError::AlreadyCreated => write!(fmt, "order has already been created"),
            OrderError::NotYetCreated => write!(fmt, "order has not been created yet"),
            OrderError::NotEditable => write!(fmt, "order can't be edited anymore"),
            OrderError::AlreadyCancelled => write!(fmt, "order has already been cancelled"),
            OrderError::AlreadyCompleted => write!(fmt, "order has already been completed"),
        }
    }
}

impl std::error::Error for OrderError {}

#[derive(Debug, Clone, Copy)]
pub struct OrderAggregate;
impl Aggregate for OrderAggregate {
    type State = OrderState;
    type Event = OrderEvent;
    type Command = OrderCommand;
    type Error = OrderError;

    fn apply_first(event: Self::Event) -> Result<Self::State, Self::Error> {
        if let OrderEvent::Created { id, at } = event {
            return Ok(OrderState::Editable {
                id,
                created_at: at,
                updated_at: at,
                items: Vec::new(),
            });
        }

        Err(OrderError::NotYetCreated)
    }

    fn apply_next(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        match event {
            OrderEvent::ItemAdded { item, at } => match state {
                OrderState::Editable {
                    id,
                    created_at,
                    items,
                    ..
                } => Ok(OrderState::Editable {
                    id,
                    created_at,
                    updated_at: at,
                    items: OrderItems::from(items).insert_or_merge(item).into(),
                }),
                _ => Err(OrderError::NotEditable),
            },
            OrderEvent::Completed { at } => match state {
                OrderState::Editable {
                    id,
                    created_at,
                    items,
                    ..
                } => Ok(OrderState::Complete {
                    id,
                    created_at,
                    completed_at: at,
                    items,
                }),
                OrderState::Complete { .. } => Err(OrderError::AlreadyCompleted),
                _ => Err(OrderError::NotEditable),
            },
            OrderEvent::Cancelled { at } => match state {
                OrderState::Editable {
                    id,
                    created_at,
                    items,
                    ..
                } => Ok(OrderState::Cancelled {
                    id,
                    created_at,
                    items,
                    cancelled_at: at,
                }),
                OrderState::Cancelled { .. } => Err(OrderError::AlreadyCancelled),
                _ => Err(OrderError::NotEditable),
            },
            _ => Err(OrderError::AlreadyCreated),
        }
    }

    fn handle_first(
        &self,
        command: Self::Command,
    ) -> BoxFuture<Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized,
    {
        Box::pin(match command {
            OrderCommand::Create { id } => {
                future::ok(vec![OrderEvent::Created { id, at: Utc::now() }])
            }
            _ => future::err(OrderError::NotYetCreated),
        })
    }

    fn handle_next<'agg, 'st: 'agg>(
        &'agg self,
        _state: &'st Self::State,
        command: Self::Command,
    ) -> BoxFuture<'agg, Result<Vec<Self::Event>, Self::Error>>
    where
        Self: Sized,
    {
        Box::pin(match command {
            OrderCommand::Create { .. } => future::err(OrderError::AlreadyCreated),
            OrderCommand::AddItem { item } => future::ok(vec![OrderEvent::ItemAdded {
                item,
                at: Utc::now(),
            }]),
            OrderCommand::Complete => future::ok(vec![OrderEvent::Completed { at: Utc::now() }]),
            OrderCommand::Cancel => future::ok(vec![OrderEvent::Cancelled { at: Utc::now() }]),
        })
    }
}
