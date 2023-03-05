use std::{error::Error as StdError, sync::Arc};

use async_trait::async_trait;
use eventually::{aggregate, command::Handler, version};
use tracing::instrument;

use crate::{
    command::{add_todo_list_item, create_todo_list},
    domain::todo,
    proto,
};

#[derive(Clone)]
pub struct TodoListService<R>
where
    R: aggregate::Repository<todo::list::List> + 'static,
{
    pub id_generator: Arc<fn() -> String>,
    pub create_todo_list: create_todo_list::Handler<R>,
    pub add_todo_list_item: add_todo_list_item::Handler<R>,
}

#[async_trait]
impl<R> proto::todo_list_service_server::TodoListService for TodoListService<R>
where
    R: aggregate::Repository<todo::list::List> + 'static,
    <R as aggregate::repository::Getter<todo::list::List>>::Error: StdError + Send + Sync + 'static,
    <R as aggregate::repository::Saver<todo::list::List>>::Error: StdError + Send + Sync + 'static,
{
    async fn create_todo_list(
        &self,
        request: tonic::Request<proto::CreateTodoListRequest>,
    ) -> Result<tonic::Response<proto::CreateTodoListResponse>, tonic::Status> {
        let request = request.into_inner();
        let todo_list_id = (self.id_generator)();

        self.create_todo_list
            .handle(
                create_todo_list::Command {
                    id: todo_list_id.clone(),
                    title: request.title,
                    owner: request.owner,
                }
                .into(),
            )
            .await
            .map(|_| tonic::Response::new(proto::CreateTodoListResponse { todo_list_id }))
            .map_err(|e| {
                use create_todo_list::Error::*;

                let error_msg = e.to_string();

                match e {
                    Create(todo::list::Error::NoOwnerSpecified | todo::list::Error::EmptyTitle) => {
                        tonic::Status::invalid_argument(error_msg)
                    }
                    Repository(err) => {
                        let conflict_error = err
                            .source()
                            .and_then(|e| e.downcast_ref::<version::ConflictError>());

                        match conflict_error {
                            None => tonic::Status::internal(error_msg),
                            Some(_) => tonic::Status::already_exists(error_msg),
                        }
                    }
                    _ => tonic::Status::internal(error_msg),
                }
            })
    }

    #[instrument(skip(self))]
    async fn get_todo_list(
        &self,
        request: tonic::Request<proto::GetTodoListRequest>,
    ) -> Result<tonic::Response<proto::GetTodoListResponse>, tonic::Status> {
        todo!()
    }

    #[instrument(skip(self))]
    async fn add_todo_item(
        &self,
        request: tonic::Request<proto::AddTodoItemRequest>,
    ) -> Result<tonic::Response<proto::AddTodoItemResponse>, tonic::Status> {
        todo!()
    }

    #[instrument(skip(self))]
    async fn toggle_todo_item(
        &self,
        request: tonic::Request<proto::ToggleTodoItemRequest>,
    ) -> Result<tonic::Response<proto::ToggleTodoItemResponse>, tonic::Status> {
        todo!()
    }

    #[instrument(skip(self))]
    async fn delete_todo_item(
        &self,
        request: tonic::Request<proto::DeleteTodoItemRequest>,
    ) -> Result<tonic::Response<proto::DeleteTodoItemResponse>, tonic::Status> {
        todo!()
    }
}
