mod api;
pub mod config;
pub mod order;
mod state;

use std::sync::Arc;

use eventually::aggregate::Optional;
use eventually::inmemory::{EventStoreBuilder, ProjectorBuilder};
use eventually::store::Select;
use eventually::{AggregateRootBuilder, Repository};

use futures::stream::StreamExt;

use tokio::sync::RwLock;

use crate::config::Config;
use crate::order::{OrderAggregate, TotalOrdersProjection};

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

    // Put the store behind an Arc to allow for clone-ness of a single instance.
    let store = Arc::new(store);

    // Create a new Projector for the desired projection.
    let mut total_orders_projector =
        ProjectorBuilder::new(store.clone(), store.clone()).build::<TotalOrdersProjection>();

    // Get a watch channel from the Projector: updates to the projector values
    // will be sent here.
    let mut total_orders_projector_rx = total_orders_projector.watch();

    // Keep the projection value in memory.
    // We can use it to access it from the context of an endpoint and serialize the read model.
    let total_orders_projection = Arc::new(RwLock::new(TotalOrdersProjection::default()));
    let total_orders_projection_state = total_orders_projection.clone();

    // Spawn a dedicated coroutine to run the projector.
    //
    // The projector will open its own running subscription, on which
    // it will receive all oldest and newest events as they come into the EventStore,
    // and it will progressively update the projection as events arrive.
    tokio::spawn(async move {
        total_orders_projector
            .run(Select::All)
            .await
            .expect("should not fail")
    });

    // Spawn a dedicated coroutine to listen to changes to the projection.
    //
    // In this case we're logging the latest version, but in more advanced
    // scenario you might want to do something more with it.
    //
    // In some cases you might not need to watch the projection changes.
    tokio::spawn(async move {
        while let Some(total_orders) = total_orders_projector_rx.next().await {
            log::info!("Total orders: {:?}", total_orders);
            *total_orders_projection_state.write().await = total_orders;
        }
    });

    // Set up the HTTP router.
    let mut app = tide::new();

    app.at("/orders").nest({
        let mut api = tide::with_state(state::AppState {
            store,
            builder: aggregate_root_builder,
            repository,
            total_orders_projection,
        });

        api.at("/history").get(api::full_history);
        api.at("/total").get(api::total_orders);

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
