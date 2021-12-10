use std::future::Future;

use async_trait::async_trait;

use crate::Message;

pub type Command<T> = Message<T>;

#[async_trait]
pub trait Handler<T>: Send + Sync {
    type Error: Send + Sync;

    async fn handle(&self, command: Command<T>) -> Result<(), Self::Error>;
}

#[async_trait]
impl<T, Err, F, Fut> Handler<T> for F
where
    T: Send + Sync + 'static,
    Err: Send + Sync,
    F: Send + Sync + Fn(Command<T>) -> Fut,
    Fut: Send + Sync + Future<Output = Result<(), Err>>,
{
    type Error = Err;

    async fn handle(&self, command: Command<T>) -> Result<(), Self::Error> {
        self(command).await
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;

    use crate::{
        aggregate, aggregate::test_user_domain::*, command, command::Command, event, event::Event,
        test,
    };

    struct CreateUser {
        email: String,
        password: String,
    }

    struct CreateUserHandler<R>(R)
    where
        R: aggregate::Repository<User, UserRoot>;

    #[async_trait]
    impl<R> command::Handler<CreateUser> for CreateUserHandler<R>
    where
        R: aggregate::Repository<User, UserRoot>,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        type Error = anyhow::Error;

        async fn handle(&self, command: Command<CreateUser>) -> Result<(), Self::Error> {
            let command = command.payload;
            let mut user = UserRoot::create(command.email, command.password)?;

            self.0.store(&mut user).await?;

            Ok(())
        }
    }

    struct ChangeUserPassword {
        email: String,
        password: String,
    }

    struct ChangeUserPasswordHandler<R>(R)
    where
        R: aggregate::Repository<User, UserRoot>;

    #[async_trait]
    impl<R> command::Handler<ChangeUserPassword> for ChangeUserPasswordHandler<R>
    where
        R: aggregate::Repository<User, UserRoot>,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        type Error = anyhow::Error;

        async fn handle(&self, command: Command<ChangeUserPassword>) -> Result<(), Self::Error> {
            let command = command.payload;

            let mut user = self.0.get(&command.email).await?;

            user.change_password(command.password)?;

            self.0.store(&mut user).await?;

            Ok(())
        }
    }

    #[tokio::test]
    async fn it_creates_a_new_user_successfully() {
        test::command_handler::Scenario
            .when(Command::from(CreateUser {
                email: "test@test.com".to_owned(),
                password: "not-a-secret".to_owned(),
            }))
            .then(vec![event::Persisted {
                stream_id: "test@test.com".to_owned(),
                version: 1,
                payload: Event::from(UserEvent::WasCreated {
                    email: "test@test.com".to_owned(),
                    password: "not-a-secret".to_owned(),
                }),
            }])
            .assert_on(|event_store| {
                CreateUserHandler(aggregate::EventSourcedRepository::from(event_store))
            })
            .await;
    }

    #[tokio::test]
    async fn it_updates_the_password_of_an_existing_user() {
        test::command_handler::Scenario
            .given(vec![event::Persisted {
                stream_id: "test@test.com".to_owned(),
                version: 1,
                payload: Event::from(UserEvent::WasCreated {
                    email: "test@test.com".to_owned(),
                    password: "not-a-secret".to_owned(),
                }),
            }])
            .when(Command::from(ChangeUserPassword {
                email: "test@test.com".to_owned(),
                password: "new-password".to_owned(),
            }))
            .then(vec![event::Persisted {
                stream_id: "test@test.com".to_owned(),
                version: 2,
                payload: Event::from(UserEvent::PasswordWasChanged {
                    password: "new-password".to_owned(),
                }),
            }])
            .assert_on(|event_store| {
                ChangeUserPasswordHandler(aggregate::EventSourcedRepository::from(event_store))
            })
            .await;
    }

    #[tokio::test]
    async fn it_fails_to_update_the_password_if_the_user_does_not_exist() {
        test::command_handler::Scenario
            .when(Command::from(ChangeUserPassword {
                email: "test@test.com".to_owned(),
                password: "new-password".to_owned(),
            }))
            .then_fails()
            .assert_on(|event_store| {
                ChangeUserPasswordHandler(aggregate::EventSourcedRepository::from(event_store))
            })
            .await;
    }
}
