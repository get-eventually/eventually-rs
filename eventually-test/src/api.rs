use chrono::{DateTime, Utc};

use eventually::store::{Persisted, Select};
use eventually::EventStore;

use futures::future::ready;
use futures::TryStreamExt;

use serde::Deserialize;

use tide::{Body, Error, Request, Response, StatusCode};

use crate::order::*;
use crate::state::*;

pub(crate) async fn full_history(req: Request<AppState>) -> Result<Response, Error> {
    #[derive(Deserialize)]
    struct Params {
        from: Option<DateTime<Utc>>,
    }

    let params: Params = req.query()?;
    let from = params.from;

    let mut stream: Vec<Persisted<String, OrderEvent>> = req
        .state()
        .store
        .stream_all(Select::All)
        .await
        .map_err(Error::from)?
        .try_filter(|event| {
            ready(match from {
                None => true,
                Some(ref from) => event.happened_at() >= from,
            })
        })
        .try_collect()
        .await?;

    stream.reverse();

    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&stream)?)
        .build())
}

pub(crate) async fn total_orders(req: Request<AppState>) -> Result<Response, Error> {
    let state = req.state().total_orders_projection.read().await;

    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&*state)?)
        .build())
}

pub(crate) async fn history(req: Request<AppState>) -> Result<Response, Error> {
    #[derive(Deserialize)]
    struct Params {
        from: Option<DateTime<Utc>>,
    }

    let id: String = req.param("id")?;
    let params: Params = req.query()?;
    let from = params.from;

    let mut stream: Vec<Persisted<String, OrderEvent>> = req
        .state()
        .store
        .stream(id, Select::All)
        .await
        .map_err(Error::from)?
        .try_filter(|event| {
            futures::future::ready(match from {
                None => true,
                Some(from) => event.happened_at() >= &from,
            })
        })
        .try_collect()
        .await?;

    stream.reverse();

    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&stream)?)
        .build())
}

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

    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&root)?)
        .build())
}

pub(crate) async fn create_order(req: Request<AppState>) -> Result<Response, Error> {
    let id: String = req.param("id")?;

    let mut root = req.state().builder.build(id);

    println!("ASD");

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

    Ok(Response::builder(StatusCode::Created)
        .body(Body::from_json(&root)?)
        .build())
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

    Ok(Response::builder(StatusCode::Accepted)
        .body(Body::from_json(&root)?)
        .build())
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

    Ok(Response::builder(StatusCode::Accepted)
        .body(Body::from_json(&root)?)
        .build())
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

    Ok(Response::builder(StatusCode::Accepted)
        .body(Body::from_json(&root)?)
        .build())
}
