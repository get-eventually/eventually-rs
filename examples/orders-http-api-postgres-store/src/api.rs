use actix_web::http::StatusCode;
use actix_web::{error, post, web, Error, HttpResponse};

use orders_domain::{OrderCommand, OrderError};

use crate::state::AppState;

#[derive(Debug, thiserror::Error)]
#[error("failed to handle order create command: {0}")]
pub struct CreateOrderError(#[source] OrderError);

impl error::ResponseError for CreateOrderError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            OrderError::AlreadyCreated => StatusCode::CONFLICT,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[post("/orders/{id}/create")]
pub(crate) async fn create_order(
    app_state: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let mut root = app_state
        .repository
        .read()
        .await
        .get(id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    root.handle(OrderCommand::Create)
        .await
        .map_err(CreateOrderError)?;

    root = app_state
        .repository
        .write()
        .await
        .add(root)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created()
        .body(serde_json::to_vec(&root).map_err(error::ErrorInternalServerError)?))
}
