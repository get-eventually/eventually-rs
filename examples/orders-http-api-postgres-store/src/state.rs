use std::sync::Arc;

use bb8_postgres::tokio_postgres::NoTls;

use eventually::optional::AsAggregate as Optional;
use eventually::sync::RwLock;
use eventually::{AggregateRootBuilder, Repository};
use eventually_postgres::EventStore;

pub(crate) type OrderAggregate = Optional<orders_domain::OrderAggregate>;
pub(crate) type OrderStore = EventStore<String, orders_domain::OrderEvent, NoTls>;
pub(crate) type OrderRepository = Repository<OrderAggregate, OrderStore>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub store: OrderStore,
    pub builder: AggregateRootBuilder<OrderAggregate>,
    pub repository: Arc<RwLock<OrderRepository>>,
    pub total_orders_projection: Arc<RwLock<orders_domain::TotalOrdersProjection>>,
}
