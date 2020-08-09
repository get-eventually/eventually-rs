mod api;
pub mod config;
pub mod order;
mod state;

use std::sync::Arc;

use eventually::aggregate::Optional;
use eventually::inmemory::EventStoreBuilder;
use eventually::{AggregateRootBuilder, Repository};

use tokio::sync::RwLock;

use crate::config::Config;
use crate::order::OrderAggregate;

pub async fn run(config: Config) -> anyhow::Result<()> {
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

    app.at("/orders").nest({
        let mut api = tide::with_state(state::AppState {
            store,
            builder: aggregate_root_builder,
            repository,
        });

        api.at("/history").get(api::full_history);

        api.at("/:id").get(api::get_order);
        api.at("/:id/create").post(api::create_order);
        api.at("/:id/add-item").post(api::add_order_item);
        api.at("/:id/complete").post(api::complete_order);
        api.at("/:id/cancel").post(api::cancel_order);
        api.at("/:id/history").get(api::history);

        api
    });

    app.listen(config.http_addr()).await?;

    Ok(())
}
