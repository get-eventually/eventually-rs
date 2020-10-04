//! A library providing components to build event-sourced applications.
//!
//! The `eventually` crate provides base abstractions to design your application
//! domain using [Domain-driven Design], using the [`Aggregate`] trait,
//! and provides a set of nice utilities around those core abstractions:
//! [`EventStore`], [`Repository`], [`Subscription`] and [`Projection`].
//!
//! ## Event Sourcing Primer
//!
//! Event Sourcing is an architectural pattern which requires domain entities'
//! (in ES lingo, **_Aggregates_**) internal state to be mutated and persisted
//! as a list of **_Domain Events_**, rather than serializing the whole state
//! to the database (as you would do with a typical CRUD architecture).
//!
//! Events are persisted in an **_Event Store_**: an ordered, append-only log of all
//! the events produced in your domain.
//!
//! Events can be retrieved from an Event Store through **_Event Streams_**,
//! stream of chronologically-ordered events.
//!
//! The state of your Aggregates can thus be rebuilt by **streaming all the Events**
//! back in the application, as they happened in time, to build the latest version
//! of the state.
//!
//! This architectural pattern brings in a lot of interesting goodies:
//! * **Auditing**: never need to guess what happened in your system,
//! thanks to the Event Store, you have a list of all the Events that have been committed
//! during time.
//! * **Time-machine**: _travel back in time_, by setting the state of your service
//! to a specific point in time, thanks to the Event Store; useful for debugging purposes.
//! * **Projections**: create read-optimized models of your Aggregates as needed
//! for you business operations, continuously, every time new Events are committed
//! to the Event Store.
//! * **Concurrency Handling**: Event-sourced application make extensive use
//! of Optimistic Concurrency to handle concurrent-writes scenarios -- concurrency
//! conflicts and state reconciliation can become part of your Domain!
//! * **High Performance, High Availability**: thanks to the append-only Event Store
//! and the use of Projections, you can write highly performant and highly available
//! services that handle an intense amount of traffic.
//!
//! [More information about this pattern can be found here.](https://docs.microsoft.com/en-us/azure/architecture/patterns/event-sourcing)
//!
//! [Domain-driven Design]: https://en.wikipedia.org/wiki/Domain-driven_design
//! [`Aggregate`]: trait.Aggregate.html
//! [`EventStore`]: trait.EventStore.html
//! [`Repository`]: struct.Repository.html
//! [`Subscription`]: trait.Subscription.html
//! [`Projection`]: trait.Projection.html

pub use eventually_core::aggregate::{
    Aggregate, AggregateExt, AggregateId, AggregateRoot, AggregateRootBuilder,
};
pub use eventually_core::projection::Projection;
pub use eventually_core::repository::Repository;
pub use eventually_core::store::EventStore;
pub use eventually_core::subscription::{EventSubscriber, Subscription};
pub use eventually_core::versioning::Versioned;
pub use eventually_util::spawn;

pub mod aggregate {
    //! Module containing support for the Aggergate pattern.
    //!
    //! ## What is an Aggregate?
    //!
    //!
    //!
    //! [Aggregate pattern]: https://martinfowler.com/bliki/DDD_Aggregate.html

    pub use eventually_core::aggregate::*;

    pub use eventually_util::optional::Aggregate as Optional;
}

pub mod versioning {
    //! Module containing support for Optimistic Concurrency using
    //! Versioning.
    //!
    //! In the future, it will contain support for conflict resolution
    //! caused by concurrent writes scenarios.

    pub use eventually_core::versioning::*;
}

pub mod repository {
    //! Repository

    pub use eventually_core::repository::*;
}

pub mod store {
    //! Event Store

    pub use eventually_core::store::*;
}

pub mod subscription {
    //! Subscription

    pub use eventually_core::subscription::*;
}

pub mod optional {
    //! Optional extension to Aggregate trait

    pub use eventually_util::optional::*;
}

pub mod inmemory {
    //! Inmemory implementations

    pub use eventually_util::inmemory::*;
}

pub mod sync {
    //! Module containing the synchronization primitives used by the library.

    pub use eventually_util::sync::*;
}

#[cfg(feature = "postgres")]
pub mod postgres {
    //! Postgres support for Event Store and Subscriptions

    pub use eventually_postgres::*;
}
