use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use eventually::{aggregate, command, message::Message};

use crate::domain::todo;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub owner: String,
}

impl Message for Command {
    fn name(&self) -> &'static str {
        "CreateTodoList"
    }
}

#[derive(Debug, Clone)]
pub struct Handler<R> {
    clock: Arc<fn() -> Instant>,
    repository: R,
}

impl<R> Handler<R>
where
    R: aggregate::repository::Saver<todo::list::List>,
{
    pub fn new(clock: fn() -> Instant, repository: R) -> Handler<R> {
        Self {
            clock: Arc::new(clock),
            repository,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<R> {
    #[error("failed to create new todo::List: {0}")]
    Create(#[source] todo::list::Error),
    #[error("failed to save new todo::List to repository: {0}")]
    Repository(#[source] R),
}

#[async_trait]
impl<R> command::Handler<Command> for Handler<R>
where
    R: aggregate::repository::Saver<todo::list::List>,
{
    type Error = Error<R::Error>;

    async fn handle(&self, envelope: command::Envelope<Command>) -> Result<(), Self::Error> {
        let cmd = envelope.message;
        let now = (self.clock)();

        let mut todo_list = todo::list::ListRoot::create(todo::list::Create {
            id: cmd.id,
            title: cmd.title,
            owner: cmd.owner,
            now,
        })
        .map_err(Error::Create)?;

        self.repository
            .save(&mut todo_list)
            .await
            .map_err(Error::Repository)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use eventually::{aggregate, command, event};
    use lazy_static::lazy_static;

    use crate::{command::create_todo_list, domain::todo};

    lazy_static! {
        static ref TEST_ID: String = String::from("new-todo-list");
        static ref TEST_TITLE: String = String::from("my-list");
        static ref TEST_OWNER: String = String::from("owner@test.com");
        static ref NOW: Instant = Instant::now();
        static ref CLOCK: fn() -> Instant = || *NOW;
    }

    #[tokio::test]
    async fn it_works() {
        command::test::Scenario
            .when(
                create_todo_list::Command {
                    id: TEST_ID.clone(),
                    title: TEST_TITLE.clone(),
                    owner: TEST_OWNER.clone(),
                }
                .into(),
            )
            .then(vec![event::Persisted {
                stream_id: TEST_ID.clone(),
                version: 1,
                event: todo::list::Event::WasCreated(todo::list::WasCreated {
                    id: TEST_ID.clone(),
                    title: TEST_TITLE.clone(),
                    owner: TEST_OWNER.clone(),
                    creation_time: *NOW,
                })
                .into(),
            }])
            .assert_on(|event_store| {
                create_todo_list::Handler::new(
                    *CLOCK,
                    aggregate::EventSourcedRepository::from(event_store),
                )
            })
            .await;
    }

    #[tokio::test]
    async fn when_todo_list_with_same_id_exists_create_command_fails() {
        command::test::Scenario
            .given(vec![event::Persisted {
                stream_id: TEST_ID.clone(),
                version: 1,
                event: todo::list::Event::WasCreated(todo::list::WasCreated {
                    id: TEST_ID.clone(),
                    title: TEST_TITLE.clone(),
                    owner: TEST_OWNER.clone(),
                    creation_time: *NOW,
                })
                .into(),
            }])
            .when(
                create_todo_list::Command {
                    id: TEST_ID.clone(),
                    title: TEST_TITLE.clone(),
                    owner: TEST_OWNER.clone(),
                }
                .into(),
            )
            .then_fails()
            .assert_on(|event_store| {
                create_todo_list::Handler::new(
                    *CLOCK,
                    aggregate::EventSourcedRepository::from(event_store),
                )
            })
            .await;
    }
}
