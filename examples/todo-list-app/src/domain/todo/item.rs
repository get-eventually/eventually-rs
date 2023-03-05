use std::time::Instant;

use eventually::{aggregate::Aggregate, message::Message};

#[derive(Debug, Clone)]
pub struct Item {
    id: String,
    title: String,
    description: Option<String>,
    completed: bool,
    due_date: Option<Instant>,
    creation_time: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasAdded {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<Instant>,
    pub creation_time: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    WasAdded(WasAdded),
}

impl Message for Event {
    fn name(&self) -> &'static str {
        panic!("it should not be called here, must be implemented on todo::list::Event!")
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum ApplyError {}

impl Aggregate for Item {
    type Id = String;
    type Event = Event;
    type Error = ApplyError;

    fn type_name() -> &'static str {
        "TodoItem"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        use Event::*;

        match (state, event) {
            (None, WasAdded(evt)) => Ok(Self {
                id: evt.id,
                title: evt.title,
                description: evt.description,
                completed: false,
                due_date: evt.due_date,
                creation_time: evt.creation_time,
            }),
            _ => todo!(),
        }
    }
}
