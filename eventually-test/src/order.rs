use std::fmt::{Display, Formatter, Result as FmtResult};

use chrono::{DateTime, Utc};

use futures::{future, future::BoxFuture};

use serde::{Deserialize, Serialize};

use eventually::optional::Aggregate;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum OrderState {
    Editable { updated_at: DateTime<Utc> },
    Complete { at: DateTime<Utc> },
    Cancelled { at: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    #[serde(skip_serializing)]
    id: String,
    created_at: DateTime<Utc>,
    items: Vec<OrderItem>,
    state: OrderState,
}

impl Order {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn items(&self) -> &Vec<OrderItem> {
        &self.items
    }

    pub fn state(&self) -> OrderState {
        self.state
    }

    pub fn is_editable(&self) -> bool {
        if let OrderState::Editable { .. } = self.state {
            return true;
        }

        false
    }
}

#[derive(Debug)]
pub enum OrderCommand {
    Create,
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
    type Id = String;
    type State = Order;
    type Event = OrderEvent;
    type Command = OrderCommand;
    type Error = OrderError;

    fn apply_first(event: Self::Event) -> Result<Self::State, Self::Error> {
        if let OrderEvent::Created { id, at } = event {
            return Ok(Order {
                id,
                created_at: at,
                items: Vec::new(),
                state: OrderState::Editable { updated_at: at },
            });
        }

        Err(OrderError::NotYetCreated)
    }

    fn apply_next(mut state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
        match event {
            OrderEvent::Created { .. } => Err(OrderError::AlreadyCreated),

            OrderEvent::ItemAdded { item, at } => {
                if let OrderState::Editable { .. } = state.state {
                    state.state = OrderState::Editable { updated_at: at };
                    state.items = OrderItems::from(state.items).insert_or_merge(item).into();
                    return Ok(state);
                }

                Err(OrderError::NotEditable)
            }

            OrderEvent::Completed { at } => {
                if let OrderState::Complete { .. } = state.state {
                    return Err(OrderError::AlreadyCompleted);
                }

                if let OrderState::Editable { .. } = state.state {
                    state.state = OrderState::Complete { at };
                    return Ok(state);
                }

                Err(OrderError::NotEditable)
            }

            OrderEvent::Cancelled { at } => {
                if let OrderState::Cancelled { .. } = state.state {
                    return Err(OrderError::AlreadyCancelled);
                }

                if let OrderState::Editable { .. } = state.state {
                    state.state = OrderState::Cancelled { at };
                    return Ok(state);
                }

                Err(OrderError::NotEditable)
            }
        }
    }

    fn handle_first<'a, 's: 'a>(
        &'a self,
        id: &'s Self::Id,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Option<Vec<Self::Event>>, Self::Error>>
    where
        Self: Sized,
    {
        Box::pin(async move {
            if let OrderCommand::Create = command {
                return Ok(Some(vec![OrderEvent::Created {
                    id: id.clone(),
                    at: Utc::now(),
                }]));
            }

            Err(OrderError::NotYetCreated)
        })
    }

    fn handle_next<'a, 's: 'a>(
        &'a self,
        _id: &'a Self::Id,
        _state: &'s Self::State,
        command: Self::Command,
    ) -> BoxFuture<'a, Result<Option<Vec<Self::Event>>, Self::Error>>
    where
        Self: Sized,
    {
        Box::pin(match command {
            OrderCommand::Create => future::err(OrderError::AlreadyCreated),
            OrderCommand::AddItem { item } => future::ok(Some(vec![OrderEvent::ItemAdded {
                item,
                at: Utc::now(),
            }])),
            OrderCommand::Complete => {
                future::ok(Some(vec![OrderEvent::Completed { at: Utc::now() }]))
            }
            OrderCommand::Cancel => {
                future::ok(Some(vec![OrderEvent::Cancelled { at: Utc::now() }]))
            }
        })
    }
}
