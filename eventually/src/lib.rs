pub use eventually_core::aggregate::{Aggregate, AggregateExt};
pub use eventually_core::store::Store;

pub mod aggregate {
    pub use eventually_core::aggregate::*;

    pub use eventually_util::aggregate::referential;
    pub use eventually_util::aggregate::referential::Aggregate as ReferentialAggregate;
}

pub mod command {
    pub use eventually_core::command::*;

    pub use eventually_util::command::dispatcher;
    pub use eventually_util::command::r#static;
}

pub mod optional {
    pub use eventually_util::optional::*;
}

pub mod versioned {
    pub use eventually_util::versioned::*;
}

// TODO: use feature-flags and include eventually-memory in its own `pub mod memory`
