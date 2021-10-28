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
//! Events are persisted in an **_Event Store_**: an ordered, append-only log of
//! all the events produced in your domain.
//!
//! Events can be retrieved from an Event Store through **_Event Streams_**,
//! stream of chronologically-ordered events.
//!
//! The state of your Aggregates can thus be rebuilt by **streaming all the
//! Events** back in the application, as they happened in time, to build the
//! latest version of the state.
//!
//! This architectural pattern brings in a lot of interesting goodies:
//! * **Auditing**: never need to guess what happened in your system,
//! thanks to the Event Store, you have a list of all the Events that have been
//! committed during time.
//! * **Time-machine**: _travel back in time_, by setting the state of your
//!   service
//! to a specific point in time, thanks to the Event Store; useful for debugging
//! purposes.
//! * **Projections**: create read-optimized models of your Aggregates as needed
//! for you business operations, continuously, every time new Events are
//! committed to the Event Store.
//! * **Concurrency Handling**: Event-sourced application make extensive use
//! of Optimistic Concurrency to handle concurrent-writes scenarios --
//! concurrency conflicts and state reconciliation can become part of your
//! Domain!
//! * **High Performance, High Availability**: thanks to the append-only Event
//!   Store
//! and the use of Projections, you can write highly performant and highly
//! available services that handle an intense amount of traffic.
//!
//! [More information about this pattern can be found here.](https://eventstore.com/blog/what-is-event-sourcing/)
//!
//! [Domain-driven Design]: https://en.wikipedia.org/wiki/Domain-driven_design
//! [`Aggregate`]: trait.Aggregate.html
//! [`EventStore`]: trait.EventStore.html
//! [`Repository`]: struct.Repository.html
//! [`Subscription`]: trait.Subscription.html
//! [`Projection`]: trait.Projection.html

#[deny(
    clippy::all,
    missing_docs,
    unsafe_code,
    unused_qualifications,
    trivial_casts
)]
pub mod aggregate;
pub use aggregate::{Aggregate, AggregateExt, AggregateId, AggregateRoot, AggregateRootFactory};

pub mod repository;
pub mod store;
pub mod versioning;
pub use store::EventStore;
pub mod projection;
pub mod subscription;
pub mod util;
pub use util::inmemory;
pub use util::optional;
pub use util::sync;
