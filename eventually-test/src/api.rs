// use chrono::{DateTime, Utc};
//
// use eventually::versioned::Versioned;
// use eventually::EventStore;
//
// use futures::StreamExt;
//
// use serde::Deserialize;

use tide::{Error, Request, Response, StatusCode};

use crate::order::*;
use crate::state::*;

// pub(crate) async fn history(req: Request<AppState>) -> Result<Response, Error> {
//     #[derive(Deserialize)]
//     struct Params {
//         from: Option<DateTime<Utc>>,
//     }
//
//     let id: String = req.param("id")?;
//     let params: Params = req.query()?;
//     let from = params.from;
//
//     let mut stream: Vec<Result<Versioned<OrderEvent>, _>> = req
//         .state()
//         .store
//         .stream(id, 0)
//         .await
//         .map_err(Error::from)?
//         .collect()
//         .await;
//
//     stream.reverse();
//
//     let mut new_stream = Vec::new();
//
//     for result in stream {
//         let event = result.map_err(Error::from)?;
//
//         if from.is_some() && event.happened_at() >= from.as_ref().unwrap() {
//             new_stream.push(event);
//         }
//     }
//
//     Response::new(StatusCode::Ok)
//         .body_json(&new_stream)
//         .map_err(Error::from)
// }

pub(crate) async fn get_order(req: Request<AppState>) -> Result<Response, Error> {
    let id: String = req.param("id")?;

    let root = req
        .state()
        .repository
        .read()
        .await
        .get(id)
        .await
        .map_err(Error::from)?;

    if root.state().is_none() {
        return Ok(Response::new(StatusCode::NotFound));
    }

    Response::new(StatusCode::Ok)
        .body_json(root.state())
        .map_err(Error::from)
}

pub(crate) async fn create_order(req: Request<AppState>) -> Result<Response, Error> {
    let id: String = req.param("id")?;

    let mut root = req.state().builder.build(id);

    root.handle(OrderCommand::Create)
        .await
        .map_err(|err| Error::new(StatusCode::BadRequest, err))?;

    root = req
        .state()
        .repository
        .write()
        .await
        .add(root)
        .await
        .map_err(Error::from)?;

    Response::new(StatusCode::Created)
        .body_json(root.state())
        .map_err(Error::from)
}

pub(crate) async fn add_order_item(mut req: Request<AppState>) -> Result<Response, Error> {
    let item: OrderItem = req.body_json().await?;

    let id: String = req.param("id")?;

    let mut root = req
        .state()
        .repository
        .read()
        .await
        .get(id)
        .await
        .map_err(Error::from)?;

    if root.state().is_none() {
        return Ok(Response::new(StatusCode::NotFound));
    }

    root.handle(OrderCommand::AddItem { item })
        .await
        .map_err(|err| Error::new(StatusCode::Forbidden, err))?;

    root = req
        .state()
        .repository
        .write()
        .await
        .add(root)
        .await
        .map_err(Error::from)?;

    Response::new(StatusCode::Accepted)
        .body_json(root.state())
        .map_err(Error::from)
}

pub(crate) async fn complete_order(req: Request<AppState>) -> Result<Response, Error> {
    let id: String = req.param("id")?;

    let mut root = req
        .state()
        .repository
        .read()
        .await
        .get(id)
        .await
        .map_err(Error::from)?;

    if root.state().is_none() {
        return Ok(Response::new(StatusCode::NotFound));
    }

    root.handle(OrderCommand::Complete)
        .await
        .map_err(|err| Error::new(StatusCode::Forbidden, err))?;

    root = req
        .state()
        .repository
        .write()
        .await
        .add(root)
        .await
        .map_err(Error::from)?;

    Response::new(StatusCode::Accepted)
        .body_json(root.state())
        .map_err(Error::from)
}

pub(crate) async fn cancel_order(req: Request<AppState>) -> Result<Response, Error> {
    let id: String = req.param("id")?;

    let mut root = req
        .state()
        .repository
        .read()
        .await
        .get(id)
        .await
        .map_err(Error::from)?;

    if root.state().is_none() {
        return Ok(Response::new(StatusCode::NotFound));
    }

    root.handle(OrderCommand::Cancel)
        .await
        .map_err(|err| Error::new(StatusCode::Forbidden, err))?;

    root = req
        .state()
        .repository
        .write()
        .await
        .add(root)
        .await
        .map_err(Error::from)?;

    Response::new(StatusCode::Accepted)
        .body_json(root.state())
        .map_err(Error::from)
}
