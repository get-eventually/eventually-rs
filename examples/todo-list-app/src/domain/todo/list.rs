use std::time::Instant;

use eventually::{
    aggregate::{Aggregate, Root},
    message::Message,
};
use eventually_macros::aggregate_root;

use crate::domain::todo;

#[derive(Debug, Clone)]
pub struct List {
    id: String,
    title: String,
    owner: String,
    items: Vec<todo::item::Item>,
    creation_time: Instant,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WasCreated {
    pub id: String,
    pub title: String,
    pub owner: String,
    pub creation_time: Instant,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    WasCreated(WasCreated),
    ItemWasAdded(todo::item::WasAdded),
}

impl Message for Event {
    fn name(&self) -> &'static str {
        match self {
            Event::WasCreated(_) => "TodoListWasCreated",
            Event::ItemWasAdded(_) => "TodoItemWasAdded",
        }
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum ApplyError {
    #[error("todo list not created yet, invalid event at this state")]
    NotCreatedYet,
    #[error("todo list already created, cannot be created again")]
    AlreadyCreated,
    #[error("failed to apply todo item domain event, {0}")]
    Item(#[from] todo::item::ApplyError),
}

impl Aggregate for List {
    type Id = String;
    type Event = Event;
    type Error = ApplyError;

    fn type_name() -> &'static str {
        "TodoList"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        use Event::*;

        match (state, event) {
            (None, WasCreated(evt)) => Ok(Self {
                id: evt.id,
                title: evt.title,
                owner: evt.owner,
                items: Vec::new(),
                creation_time: evt.creation_time,
            }),
            (Some(_), WasCreated(_)) => Err(ApplyError::AlreadyCreated),
            (None, _) => Err(ApplyError::NotCreatedYet),
            (Some(mut list), ItemWasAdded(evt)) => {
                let item = todo::item::Item::apply(None, todo::item::Event::WasAdded(evt))?;
                list.items.push(item);
                Ok(list)
            }
        }
    }
}

#[aggregate_root(List)]
#[derive(Debug, Clone)]
pub struct ListRoot;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("the provided id is empty")]
    EmptyId,
    #[error("the title specified is empty")]
    EmptyTitle,
    #[error("no owner has been specified")]
    NoOwnerSpecified,
    #[error("the provided item id is empty")]
    EmptyItemId,
    #[error("the title for the item specified is empty")]
    EmptyItemTitle,
    #[error("item already exists")]
    ItemAlreadyExists,
    #[error("failed to record domain event: {0}")]
    RecordDomainEvent(#[from] ApplyError),
}

#[derive(Debug)]
pub struct Create {
    pub id: String,
    pub title: String,
    pub owner: String,
    pub now: Instant,
}

impl ListRoot {
    pub fn create(input: Create) -> Result<Self, Error> {
        if input.id.is_empty() {
            return Err(Error::EmptyId);
        }
        if input.title.is_empty() {
            return Err(Error::EmptyTitle);
        }
        if input.owner.is_empty() {
            return Err(Error::NoOwnerSpecified);
        }

        let event = Event::WasCreated(WasCreated {
            id: input.id,
            title: input.title,
            owner: input.owner,
            creation_time: input.now,
        });

        Ok(Root::record_new(event.into()).map(Self)?)
    }
}

#[derive(Debug)]
pub struct AddItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<Instant>,
    pub now: Instant,
}

impl ListRoot {
    pub fn add_item(&mut self, input: AddItem) -> Result<(), Error> {
        if input.id.is_empty() {
            return Err(Error::EmptyItemId);
        }
        if input.title.is_empty() {
            return Err(Error::EmptyItemTitle);
        }

        let item_already_exists = self
            .items
            .iter()
            .any(|item| item.aggregate_id() == &input.id);

        if item_already_exists {
            return Err(Error::ItemAlreadyExists);
        }

        let event = Event::ItemWasAdded(todo::item::WasAdded {
            id: input.id,
            title: input.title,
            description: input.description,
            due_date: input.due_date,
            creation_time: input.now,
        });

        Ok(self.record_that(event.into())?)
    }
}
