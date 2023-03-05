use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use eventually::{aggregate, command, message::Message};

use crate::domain::todo;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Command {
    pub todo_list_id: String,
    pub todo_item_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<Instant>,
}

impl Message for Command {
    fn name(&self) -> &'static str {
        "AddTodoListItem"
    }
}

#[derive(Debug, Clone)]
pub struct Handler<R> {
    clock: Arc<fn() -> Instant>,
    repository: R,
}

impl<R> Handler<R>
where
    R: aggregate::repository::Repository<todo::list::List>,
{
    pub fn new(clock: fn() -> Instant, repository: R) -> Handler<R> {
        Self {
            clock: Arc::new(clock),
            repository,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<GetErr, SaveErr> {
    #[error("failed to create add new item to todo::List: {0}")]
    AddItem(#[source] todo::list::Error),
    #[error("failed to get the todo::List from repository: {0}")]
    Get(#[source] aggregate::repository::GetError<GetErr>),
    #[error("failed to save new todo::List to repository: {0}")]
    Save(#[source] SaveErr),
}

#[async_trait]
impl<R> command::Handler<Command> for Handler<R>
where
    R: aggregate::repository::Repository<todo::list::List>,
{
    type Error = Error<
        <R as aggregate::repository::Getter<todo::list::List>>::Error,
        <R as aggregate::repository::Saver<todo::list::List>>::Error,
    >;

    async fn handle(&self, envelope: command::Envelope<Command>) -> Result<(), Self::Error> {
        let cmd = envelope.message;
        let now = (self.clock)();

        let mut todo_list = self
            .repository
            .get(&cmd.todo_list_id)
            .await
            .map(todo::list::ListRoot::from)
            .map_err(Error::Get)?;

        todo_list
            .add_item(todo::list::AddItem {
                id: cmd.todo_item_id,
                title: cmd.title,
                description: cmd.description,
                due_date: cmd.due_date,
                now,
            })
            .map_err(Error::AddItem)?;

        self.repository
            .save(&mut todo_list)
            .await
            .map_err(Error::Save)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use eventually::{aggregate, command, event};
    use lazy_static::lazy_static;

    use crate::{command::add_todo_list_item, domain::todo};

    lazy_static! {
        static ref TEST_TODO_LIST_ID: String = String::from("new-todo-list");
        static ref TEST_TITLE: String = String::from("my-list");
        static ref TEST_OWNER: String = String::from("owner@test.com");
        static ref TEST_TODO_ITEM_ID: String = String::from("new-todo-item");
        static ref TEST_TODO_ITEM_TITLE: String = String::from("do something please");
        static ref NOW: Instant = Instant::now();
        static ref CLOCK: fn() -> Instant = || *NOW;
    }

    #[tokio::test]
    async fn it_works() {
        command::test::Scenario
            .given(vec![event::Persisted {
                stream_id: TEST_TODO_LIST_ID.clone(),
                version: 1,
                event: todo::list::Event::WasCreated(todo::list::WasCreated {
                    id: TEST_TODO_LIST_ID.clone(),
                    title: TEST_TITLE.clone(),
                    owner: TEST_OWNER.clone(),
                    creation_time: *NOW,
                })
                .into(),
            }])
            .when(
                add_todo_list_item::Command {
                    todo_list_id: TEST_TODO_LIST_ID.clone(),
                    todo_item_id: TEST_TODO_ITEM_ID.clone(),
                    title: TEST_TITLE.clone(),
                    description: None,
                    due_date: None,
                }
                .into(),
            )
            .then(vec![event::Persisted {
                stream_id: TEST_TODO_LIST_ID.clone(),
                version: 2,
                event: todo::list::Event::ItemWasAdded(todo::item::WasAdded {
                    id: TEST_TODO_ITEM_ID.clone(),
                    title: TEST_TITLE.clone(),
                    description: None,
                    due_date: None,
                    creation_time: *NOW,
                })
                .into(),
            }])
            .assert_on(|event_store| {
                add_todo_list_item::Handler::new(
                    *CLOCK,
                    aggregate::EventSourcedRepository::from(event_store),
                )
            })
            .await;
    }

    #[tokio::test]
    async fn it_fails_when_adding_twice_the_same_todo_item() {
        command::test::Scenario
            .given(vec![
                event::Persisted {
                    stream_id: TEST_TODO_LIST_ID.clone(),
                    version: 1,
                    event: todo::list::Event::WasCreated(todo::list::WasCreated {
                        id: TEST_TODO_LIST_ID.clone(),
                        title: TEST_TITLE.clone(),
                        owner: TEST_OWNER.clone(),
                        creation_time: *NOW,
                    })
                    .into(),
                },
                event::Persisted {
                    stream_id: TEST_TODO_LIST_ID.clone(),
                    version: 2,
                    event: todo::list::Event::ItemWasAdded(todo::item::WasAdded {
                        id: TEST_TODO_ITEM_ID.clone(),
                        title: TEST_TITLE.clone(),
                        description: None,
                        due_date: None,
                        creation_time: *NOW,
                    })
                    .into(),
                },
            ])
            .when(
                add_todo_list_item::Command {
                    todo_list_id: TEST_TODO_LIST_ID.clone(),
                    todo_item_id: TEST_TODO_ITEM_ID.clone(),
                    title: TEST_TITLE.clone(),
                    description: None,
                    due_date: None,
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| {
                add_todo_list_item::Handler::new(
                    *CLOCK,
                    aggregate::EventSourcedRepository::from(event_store),
                )
            })
            .await;
    }
}
