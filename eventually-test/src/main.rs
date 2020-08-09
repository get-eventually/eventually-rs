mod api;
mod config;
mod order;
mod state;

use std::sync::Arc;

use envconfig::Envconfig;

use eventually::aggregate::Optional;
use eventually::inmemory::EventStoreBuilder;
use eventually::{AggregateRootBuilder, Repository};

use tokio::sync::RwLock;

use crate::config::Config;
use crate::order::OrderAggregate;

fn main() -> anyhow::Result<()> {
    smol::run(run())
}

async fn run() -> anyhow::Result<()> {
    let config = Config::init()?;

    femme::with_level(config.log_level);

    // Aggregate target: in this case it's empty, but usually it would use
    // some domain services or internal repositories.
    let aggregate = OrderAggregate.as_aggregate();

    // Event store for the OrderAggregate.
    let store = EventStoreBuilder::for_aggregate(&aggregate);

    // Builder for all new AggregateRoot instances.
    let aggregate_root_builder = AggregateRootBuilder::from(Arc::new(aggregate));

    // Creates a Repository to read and store OrderAggregates.
    let repository = Arc::new(RwLock::new(Repository::new(
        aggregate_root_builder.clone(),
        store.clone(),
    )));

    // Set up the HTTP router.
    let mut app = tide::new();

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
        api.at("/history").get(api::history);

        api
    });

    app.listen(config.http_addr()).await?;

    Ok(())
}
