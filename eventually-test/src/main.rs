mod api;
mod config;
mod log;
mod order;
mod state;

use std::sync::Arc;

use envconfig::Envconfig;

use eventually::aggregate::Optional;
use eventually::{AggregateRootBuilder, Repository};
use eventually_postgres::EventStoreBuilder;

use tokio::sync::RwLock;

use crate::config::Config;
use crate::order::OrderAggregate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::init()?;

    env_logger::builder().filter_level(config.log_level).init();

    // Aggregate target: in this case it's empty, but usually it would use
    // some domain services or internal repositories.
    let aggregate = OrderAggregate.as_aggregate();

    // Open a connection with Postgres.
    let (client, connection) =
        tokio_postgres::connect(&config.postgres_dsn(), tokio_postgres::NoTls)
            .await
            .map_err(|err| {
                eprintln!("failed to connect to Postgres: {}", err);
                err
            })?;

    // The connection, responsible for the actual IO, must be handled by a different
    // execution context.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Use an EventStoreBuilder to build multiple EventStore instances.
    let event_store_builder = EventStoreBuilder::from(Arc::new(RwLock::new(client)));

    // Event store for the OrderAggregate.
    let store = {
        let store = event_store_builder.aggregate_stream(&aggregate, "orders");
        store.create_stream().await?;
        store
    };

    // Builder for all new AggregateRoot instances.
    let aggregate_root_builder = AggregateRootBuilder::from(Arc::new(aggregate));

    // Creates a Repository to read and store OrderAggregates.
    let repository = Arc::new(RwLock::new(Repository::new(
        aggregate_root_builder.clone(),
        store.clone(),
    )));

    // Set up the HTTP router.
    let mut app = tide::new();

    app.middleware(log::Middleware::new());

    app.at("/orders/:id").nest({
        let mut api = tide::with_state(state::AppState {
            store,
            builder: aggregate_root_builder,
            repository,
        });

        api.at("/").get(api::get_order);
        api.at("/create").post(api::create_order);
        api.at("/add-item").post(api::add_order_item);
        api.at("/complete").post(api::complete_order);
        api.at("/cancel").post(api::cancel_order);
        // api.at("/history").get(api::history);

        api
    });

    app.listen(config.http_addr()).await?;

    Ok(())
}
