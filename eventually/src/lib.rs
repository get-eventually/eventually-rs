pub use eventually_core::aggregate::{
    Aggregate, AggregateExt, AggregateId, AggregateRoot, Identifiable,
};
pub use eventually_core::repository::Repository;
pub use eventually_core::store::EventStore;

pub mod aggregate {
    pub use eventually_core::aggregate::*;

    pub use eventually_util::optional::Aggregate as Optional;
    pub use eventually_util::versioned::AsAggregate as Versioned;
}

pub mod store {
    pub use eventually_core::store::*;
}

pub mod optional {
    pub use eventually_util::optional::*;
}

pub mod versioned {
    pub use eventually_util::versioned::*;
}

pub mod inmemory {
    pub use eventually_util::inmemory::*;
}
