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
//! [More information about this pattern can be found here.](https://eventstore.com/blog/what-is-event-sourcing/)
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
pub use eventually_util::inmemory::Projector;
pub use eventually_util::spawn;

pub mod aggregate {
    //! Module containing support for the Aggergate pattern.
    //!
    //! ## What is an Aggregate?
    //!
    //! An [Aggregate] is the most important concept in your domain.
    //!
    //! It represents the entities your business domain is composed of,
    //! and the business logic your domain is exposing.
    //!
    //! For example: in an Order Management bounded-context (e.g. a microservice),
    //! the concepts of Order or Customer are two potential [Aggregate]s.
    //!
    //! Aggregates expose mutations with the concept of **commands**:
    //! from the previous example, an Order might expose some commands such as
    //! _"Add Order Item"_, or _"Remove Order Item"_, or _"Place Order"_
    //! to close the transaction.
    //!
    //! In Event Sourcing, the Aggregate state is modified by the usage of
    //! **Domain Events**, which carry some or all the fields in the state
    //! in a certain logical meaning.
    //!
    //! As such, commands in Event Sourcing will **produce** Domain Events.
    //!
    //! Aggregates should provide a way to **fold** Domain Events on the
    //! current value of the state, to produce the next state.
    //!
    //! ## Aggregate in `eventually`
    //!
    //! `eventually` supports the Aggregates through the [`Aggregate`] trait.
    //!
    //! An [`Aggregate`] allows to:
    //! * specify the available commands through the [`Command`] associated type,
    //! usually an `enum`,
    //! * define how to handle commands through the [`handle`] method,
    //! * specify the Aggregate's Domain Events through the [`Event`]
    //! associated type, usually an `enum`,
    //! * define the transition logic from one state to another by an [`Event`],
    //! through the [`apply`] method.
    //!
    //! An [`Aggregate`] is essentialy an _actor_, which means it should contain
    //! all the dependencies it needs to execute commands (e.g. repositories,
    //! external clients, validators, etc...)
    //!
    //! Let's take the Orders Management example:
    //!
    //! ```rust
    //! use eventually::Aggregate;
    //! use futures::future::BoxFuture;
    //!
    //! // This is the state of our Order, which contains one or more items,
    //! // and whether it has been placed or not.
    //! #[derive(Default)]
    //! struct OrderState {
    //!     items: Vec<OrderItem>,
    //!     placed: bool,
    //! }
    //!
    //! // This is a single order item, which contains the order SKU, the number
    //! // of quantities of the same item and the price.
    //! struct OrderItem {
    //!     sku: String,
    //!     quantity: u32,
    //!     price: f32,
    //! }
    //!
    //! // Let's say we have a catalog of available items in our system.
    //! //
    //! // We want to make sure that each item that has been added to the Order
    //! // is available in the catalog.
    //! //
    //! // We're going to use this trait in the Aggregate implementation,
    //! // when handling events to add OrderItems.
    //! trait OrderItemsCatalog {
    //!     fn get_item(&self, sku: String) -> BoxFuture<OrderItem>;
    //! }
    //!
    //! // This is the list of all the commands that are available on our Aggregate.
    //! enum OrderCommand {
    //!     // Make the order final and place it to the production line.
    //!     PlaceOrder,
    //!     // Add an OrderItem to the Order, specifying a quantity.
    //!     AddOrderItem {
    //!         sku: String,
    //!         quantity: u32,
    //!     },
    //!     // Remove an OrderItem from the Order: the item is removed
    //!     // when the quantity in the Order is less or equal than
    //!     // the quantity asked to be removed by the command.
    //!     RemoveOrderItem {
    //!         sku: String,
    //!         quantity: u32,
    //!     },
    //! }
    //!
    //! // This is the list of all the domain events defined on the Order aggregate.
    //! enum OrderEvent {
    //!     // The list of OrderItems is updated with the specified value.
    //!     OrderItemsUpdated {
    //!         items: Vec<OrderItem>,
    //!     },
    //!     // Marks the order as placed.
    //!     OrderPlaced,
    //! }
    //!
    //! // This is our Aggregate actor, which will contain all the necessary
    //! // dependencies for handling commands.
    //! //
    //! // In this case, it will only contain the OrderItemCatalog.
    //! struct OrderAggregate {
    //!     catalog: std::sync::Arc<dyn OrderItemsCatalog>,
    //! }
    //!
    //! // Implementation for the Aggregate trait.
    //! impl Aggregate for OrderAggregate {
    //!     type Id = u64;
    //!     type State = OrderState;
    //!     type Event = OrderEvent;
    //!     type Command = OrderCommand;
    //!     type Error = std::convert::Infallible; // This should be a meaningful error.
    //!
    //!     fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error> {
    //!         unimplemented!()
    //!     }
    //!
    //!     fn handle<'a, 's: 'a>(
    //!         &'a self,
    //!         id: &'s Self::Id,
    //!         state: &'s Self::State,
    //!         command: Self::Command,
    //!     ) -> BoxFuture<'a, Result<Option<Vec<Self::Event>>, Self::Error>>
    //!     where
    //!         Self: Sized
    //!     {
    //!         unimplemented!()
    //!     }
    //! }
    //! ```
    //!
    //! ### Note on [`State`]
    //!
    //! An [`Aggregate`]'s [`State`] type needs to implement the `Default` trait,
    //! to always have an initial state representation.
    //!
    //! This is very important for functional _folding_ of [`Event`]s
    //! done by [`apply`].
    //!
    //! A common type used in [`State`] is `Option<StateType>`, where
    //! `StateType` doesn't usually implement `Default`. This is to represent
    //! a nullable state before the first command is received.
    //!
    //! Given the common use-case, `eventually` has included support for
    //! the [`Optional`] aggregate trait, where you can use `StateType`
    //! directly in the `State` associated type.
    //!
    //! [`Optional`] is compatible with the [`Aggregate`] trait through
    //! the [`as_aggregate`] method, or the [`optional::AsAggregate`]
    //! newtype adapter.
    //!
    //! ### Interacting with Aggregates using `AggregateRoot`
    //!
    //! Interaction with Aggregates is doable through an [`AggregateRoot`].
    //!
    //! An [`AggregateRoot`] represents a specific Aggregate instance,
    //! by managing its [`State`] and the access to submit commands.
    //!
    //! Access to an [`AggregateRoot`] is obtainable in two ways:
    //! 1. Through the [`AggregateRootBuilder`], useful for testing
    //! 2. Through a [`Repository`] instance
    //!
    //! More on the [`Repository`] in the [module-level documentation](../repository/index.html).
    //!
    //! [Aggregate]: https://martinfowler.com/bliki/DDD_Aggregate.html
    //! [`Aggregate`]: trait.Aggregate.html
    //! [`Command`]: trait.Aggregate.html#associatedType.Command
    //! [`Event`]: trait.Aggregate.html#associatedType.Event
    //! [`State`]: trait.Aggregate.html#associatedType.State
    //! [`handle`]: trait.Aggregate.html#tymethod.handle
    //! [`apply`]: trait.Aggregate.html#tymethod.apply
    //! [`Optional`]: trait.Optional.html
    //! [`as_aggregate`]: trait.Optional.html#method.as_aggregate
    //! [`optional::AsAggregate`]: ../optional/struct.AsAggregate.html
    //! [`AggregateRoot`]: struct.AggregateRoot.html
    //! [`AggregateRootBuilder`]: struct.AggregateRootBuilder.html
    //! [`Repository`]: struct.Repository.html

    pub use eventually_core::aggregate::*;

    pub use eventually_core::repository::Repository;
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
    //! Module containing Repository implementation to retrieve,
    //! save and delete Aggregates.
    //!
    //! ## Repository and Aggregates
    //!
    //! As described in the [Interacting with Aggregates using `AggregateRoot`]
    //! section in the `aggregate` module-level documentation, in order to
    //! interact with an Aggregate instance you need to use an [`AggregateRoot`].
    //!
    //! To get an [`AggregateRoot`], you can use a [`Repository`] instance.
    //!
    //! The [`Repository`] allows to **retrieve**, **save** and **remove**
    //! specific Aggregate instances, by using an underlying [`EventStore`]
    //! implementation that handles the Aggregate's events.
    //!
    //! A [`Repository`] will **always** return an [`AggregateRoot`] instance
    //! on read, whether or not events are present in the [`EventStore`].
    //!
    //! Use the [`Repository`] to implement your bounded-context application logic,
    //! for example in HTTP or RPC handlers.
    //!
    //! [Interacting with Aggregates using `AggregateRoot`]: ../aggregate/index.html#interacting-with-aggregates-using-aggregateroot
    //! [`AggregateRoot`]: ../aggregate/struct.AggregateRoot.html
    //! [`Repository`]: struct.Repository.html
    //! [`EventStore`]: ../store/trait.EventStore.html

    pub use eventually_core::repository::*;
}

pub mod store {
    //! Module containing support for the Event Store.
    //!
    //! ## Event Store and `eventually`
    //!
    //! An Event Store is an ordered, append-only log of many Domain Events
    //! related to one or more Aggregate instances.
    //!
    //! The Domain Events committed to the Event Store can be streamed back into
    //! the application to load the latest value of an Aggregate state,
    //! by using the concept of Event Streams.
    //!
    //! Since Domain Events in the Store are ordered both globally and
    //! on an Aggregate-instance basis, the Event Stream can either be global
    //! or related to a single Aggregate.
    //!
    //! `eventually` adds support for the Event Store through the [`EventStore`]
    //! trait.
    //!
    //! You rarely need to use the [`EventStore`] directly when writing your
    //! application, since `eventually` exposes multiple utilities that
    //! instrument the usage of the store for you. For example:
    //! * [`Repository`] for retrieving, saving and deleting Aggregates
    //! * [`Projector`] to run [`Projection`]s (check out the [`projection` module documentation])
    //!
    //! [`EventStore`]: trait.EventStore.html
    //! [`Repository`]: ../repository/struct.Repository.html
    //! [`Projector`]: ../struct.Projector.html
    //! [`Projection`]: ../trait.Projection.html
    //! [`projection` module documentation]: ../projection/index.html

    pub use eventually_core::store::*;
}

pub mod subscription {
    //! Module containing support for Subscriptions to Events coming
    //! from the Event Store.
    //!
    //! ## What are Subscriptions?
    //!
    //! Subscriptions, as the name suggest, allows to subscribe to changes
    //! in the Event Store. Essentialy, Subscriptions receive Events
    //! when they get committed to the Store.
    //!
    //! This allows for near real-time processing of multiple things, such as
    //! publishing committed events on a message broker, or running **projections**
    //! (more on that on [`Projection` documentation]).
    //!
    //! ## Subscriptions in `eventually`
    //!
    //! ### `EventSubscriber` trait
    //!
    //! In order to subscribe to Events, `eventually` exposes the [`EventSubscriber`]
    //! trait, usually implemented by [`EventStore`] implementations.
    //!
    //! An [`EventSubscriber`] opens an _endless_ [`EventStream`], that gets
    //! closed only at application shutdown, or if the stream gets explicitly dropped.
    //!
    //! The [`EventStream`] receives all the **new Events** committed
    //! to the [`EventStore`].
    //!
    //! ### `Subscription` trait
    //!
    //! The [`Subscription`] trait represent an ongoing subscription
    //! to Events coming from an [`EventStream`], as described above.
    //!
    //! Similarly to the [`EventSubscriber`], a [`Subscription`]
    //! returns an _endless_ stream of Events called [`SubscriptionStream`].
    //!
    //! However, [`Subscription`]s are **stateful**: they save the latest
    //! Event sequence number that has been processed through the [`SubscriptionStream`],
    //! by using the [`checkpoint`] method. Later, the [`Subscription`] can be
    //! restarted from where it was left off using the [`resume`] method.
    //!
    //! This module exposes a simple [`Subscription`] implementation:
    //! [`Transient`], for in-memory, one-off subscriptions.
    //!
    //! For a long-running [`Subscription`] implementation,
    //! take a look at persisted subscriptions, such as [`postgres::subscription::Persisted`].
    //!
    //! [`Projection` documentation]: ../trait.Projection.html
    //! [`EventSubscriber`]: trait.EventSubscriber.html
    //! [`EventStore`]: ../store/trait.EventStore.html
    //! [`EventStream`]: type.EventStream.html
    //! [`Subscription`]: trait.Subscription.html
    //! [`SubscriptionStream`]: type.SubscriptionStream.html
    //! [`checkpoint`]: trait.Subscription.html#tymethod.checkpoint
    //! [`resume`]: trait.Subscription.html#tymethod.resume
    //! [`Transient`]: struct.Transient.html
    //! [`postgres::subscription::Persisted`]: ../postgres/subscription/struct.Persisted.html

    pub use eventually_core::subscription::*;
}

pub mod optional {
    //! Module for the Aggregate extension trait using an `Option` state.
    //!
    //! For more information, take a look at the [relevant `aggregate` documentation section]
    //! on the [`Optional`] aggregate trait.
    //!
    //! [relevant `aggregate` documentation section]: ../aggregate/index.html#note-on-state
    //! [`Optional`]: trait.Aggregate.html

    pub use eventually_util::optional::*;
}

pub mod inmemory {
    //! Module containing utilities using in-memory backend strategy.

    pub use eventually_util::inmemory::*;
}

pub mod sync {
    //! Module containing the synchronization primitives used by the library.

    pub use eventually_util::sync::*;
}

#[cfg(feature = "postgres")]
pub mod postgres {
    //! Module containing Event Store support using PostgreSQL backend.
    //!
    //! ## Event Store
    //!
    //! The module contains an [`eventually::EventStore`] trait implementation
    //! by using an [`EventStore`] instance.
    //!
    //! In order to create an [`EventStore`] instance, you need to use
    //! the [`EventStoreBuilder`] object.
    //!
    //! Here you can find a complete example on how to initialize the
    //! [`EventStore`], ready to be used in an [`eventually::Repository`] instance:
    //!
    //! ```no_run
    //! # use std::sync::Arc;
    //! # use eventually_postgres::EventStoreBuilder;
    //! #
    //! # async fn dox() -> Result<(), Box<dyn std::error::Error>> {
    //! // Open a connection with Postgres.
    //! let (mut client, connection) =
    //!     tokio_postgres::connect("postgres://user@pass:localhost:5432/db", tokio_postgres::NoTls)
    //!         .await
    //!         .map_err(|err| {
    //!             eprintln!("failed to connect to Postgres: {}", err);
    //!             err
    //!         })?;
    //!
    //! // The connection, responsible for the actual IO, must be handled by a different
    //! // execution context.
    //! //
    //! // NOTE: if this connection fails with Err(e), then the application
    //! // can't read nor write from the database anymore.
    //! tokio::spawn(async move {
    //!     if let Err(e) = connection.await {
    //!         eprintln!("connection error: {}", e);
    //!     }
    //! });
    //!
    //! // A domain event example -- it is deliberately simple.
    //! #[derive(Debug, Clone)]
    //! struct SomeEvent;
    //!
    //! // Use an EventStoreBuilder to build multiple EventStore instances.
    //! //
    //! // The only way to get an EventStoreBuilder instance is to use
    //! // migrate_database, to ensure the database schemas used by Eventually
    //! // are using the latest versions.
    //! let builder = EventStoreBuilder::migrate_database(&mut client)
    //!     .await?
    //!     // To build an EventStore instance, the client must be wrapped
    //!     // around an Arc in order to be shared among multiple threads.
    //!     .builder(Arc::new(client));
    //!
    //! // Event store for the events.
    //! //
    //! // When building an new EventStore instance, a type name is always needed
    //! // to distinguish between different aggregates.
    //! //
    //! // You can also use std::any::type_name for that.
    //! let store = builder.build::<String, SomeEvent>("aggregate-name").await?;
    //!
    //! # Ok(())
    //! # }
    //! ```
    //!
    //! ## Subscribing to Events
    //!
    //! In order to subscribe to committed events and to run projections,
    //! the module exposes an [`eventually::EventSubscriber`] trait implementation
    //! using the [`EventSubscriber`] type.
    //!
    //! [`eventually::EventStore`]: ../trait.EventStore.html
    //! [`EventStore`]: struct.EventStore.html
    //! [`EventStoreBuilder`]: struct.EventStoreBuilder.html
    //! [`eventually::Repository`]: ../struct.Repository.html
    //! [`eventually::EventSubscriber`]: ../trait.EventSubscriber.html
    //! [`EventSubscriber`]: struct.EventSubscriber.html

    pub use eventually_postgres::*;
}
