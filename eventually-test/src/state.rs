use std::sync::Arc;

use eventually::optional::AsAggregate as Optional;
use eventually::{AggregateRootBuilder, Repository};
use eventually_postgres::EventStore;

use tokio::sync::RwLock;

use crate::order;

pub(crate) type OrderAggregate = Optional<order::OrderAggregate>;
pub(crate) type OrderStore = EventStore<String, order::OrderEvent>;
pub(crate) type OrderRepository = Repository<OrderAggregate, OrderStore>;

pub(crate) struct AppState {
    pub store: OrderStore,
    pub builder: AggregateRootBuilder<OrderAggregate>,
    pub repository: Arc<RwLock<OrderRepository>>,
}
