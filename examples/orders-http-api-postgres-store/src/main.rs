#![allow(missing_docs)]

mod api;
mod config;
mod state;

use std::sync::Arc;

use actix_web::{App, HttpServer};

use bb8::Pool;
use bb8_postgres::tokio_postgres::NoTls;
use bb8_postgres::PostgresConnectionManager;

use envconfig::Envconfig;

use eventually::aggregate::Optional;
use eventually::inmemory::Projector;
use eventually::subscription::Transient as TransientSubscription;
use eventually::sync::RwLock;
use eventually::{AggregateRootBuilder, Repository};
use eventually_postgres::{EventStoreBuilder, EventSubscriber};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use orders_domain::{OrderAggregate, OrderEvent, TotalOrdersProjection};

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;

    let config = config::Config::init_from_env()?;

    // Build the connection pool for communication with Postgres.
    let pg_manager = PostgresConnectionManager::new_from_stringlike(config.postgres_dsn(), NoTls)?;
    let pool = Pool::builder().build(pg_manager).await?;

    // Event store for the OrderAggregate.
    let store = EventStoreBuilder::migrate_database(pool.clone())
        .await?
        .builder(pool)
        .build::<String, OrderEvent>("orders")
        .await?;

    let subscriber = EventSubscriber::new(&config.postgres_dsn(), "orders").await?;

    // Aggregate target: in this case it's empty, but usually it would use
    // some domain services or internal repositories.
    let aggregate = OrderAggregate.as_aggregate();

    // Builder for all new AggregateRoot instances.
    let aggregate_root_builder = AggregateRootBuilder::from(aggregate);

    // Creates a Repository to read and store OrderAggregates.
    let repository = Arc::new(RwLock::new(Repository::new(
        aggregate_root_builder.clone(),
        store.clone(),
    )));

    // Create a new in-memory projection to keep the total orders computed by
    // the application.
    let total_orders_projection = Arc::new(RwLock::new(TotalOrdersProjection::default()));

    // Create an in-memory transient Subscription that starts from the very
    // beginning of the EventStream.
    let subscription = TransientSubscription::new(store.clone(), subscriber);

    // Create a new Projector for the desired projection.
    let mut total_orders_projector = Projector::new(total_orders_projection.clone(), subscription);

    // Spawn a dedicated coroutine to run the projector.
    //
    // The projector will open its own running subscription, on which
    // it will receive all oldest and newest events as they come into the EventStore,
    // and it will progressively update the projection as events arrive.
    eventually::spawn(async move { total_orders_projector.run().await.expect("should not fail") });

    let app_state = state::AppState {
        store,
        builder: aggregate_root_builder,
        repository,
        total_orders_projection,
    };

    Ok(HttpServer::new(move || {
        App::new()
            .data(app_state.clone())
            .service(api::create_order)
    })
    .bind(config.addr())?
    .run()
    .await?)
}

fn init_tracing() -> anyhow::Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .try_init()?;

    Ok(())
}
