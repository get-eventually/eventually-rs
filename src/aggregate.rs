//! Module containing support for the Aggergate pattern.
//!
//! ## What is an Aggregate?
//!
//! An [Aggregate] is the most important concept in your domain.
//!
//! It represents the entities your business domain is composed of,
//! and the business logic your domain is exposing.
//!
//! For example: in an Order Management bounded-context (e.g. a
//! microservice), the concepts of Order or Customer are two potential
//! [Aggregate]s.
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
//! * specify the available commands through the [`Command`] associated
//!   type,
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
//! use async_trait::async_trait;
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
//! trait OrderItemsCatalog: Send + Sync {
//!     fn get_item(&self, sku: String) -> BoxFuture<OrderItem>;
//! }
//!
//! // This is the list of all the commands that are available on our Aggregate.
//! enum OrderCommand {
//!     // Make the order final and place it to the production line.
//!     PlaceOrder,
//!     // Add an OrderItem to the Order, specifying a quantity.
//!     AddOrderItem { sku: String, quantity: u32 },
//!     // Remove an OrderItem from the Order: the item is removed
//!     // when the quantity in the Order is less or equal than
//!     // the quantity asked to be removed by the command.
//!     RemoveOrderItem { sku: String, quantity: u32 },
//! }
//!
//! // This is the list of all the domain events defined on the Order aggregate.
//! enum OrderEvent {
//!     // The list of OrderItems is updated with the specified value.
//!     OrderItemsUpdated { items: Vec<OrderItem> },
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
//! #[async_trait]
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
//!     async fn handle(
//!         &self,
//!         id: &Self::Id,
//!         state: &Self::State,
//!         command: Self::Command,
//!     ) -> Result<Vec<Self::Event>, Self::Error>
//!     {
//!         unimplemented!()
//!     }
//! }
//! ```
//!
//! ### Note on [`State`]
//!
//! An [`Aggregate`]'s [`State`] type needs to implement the `Default`
//! trait, to always have an initial state representation.
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
//! 1. Through the [`AggregateRootFactory`], useful for testing
//! 2. Through a [`Repository`] instance
//!
//! More on the [`Repository`] in the [module-level
//! documentation](../repository/index.html).
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
//! [`AggregateRootFactory`]: struct.AggregateRootFactory.html
//! [`Repository`]: struct.Repository.html

use std::fmt::Debug;
use std::ops::Deref;

use async_trait::async_trait;

pub use crate::optional::Aggregate as Optional;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::versioning::Versioned;

/// A short extractor type for the [`Aggregate`] [`Id`](Aggregate::Id).
pub type AggregateId<A> = <A as Aggregate>::Id;

/// An [`Aggregate`] manages a domain entity [`State`](Aggregate::State), acting
/// as a _transaction boundary_.
///
/// It allows **state mutations** through the use of
/// [`Command`](Aggregate::Command)s, which the Aggregate instance handles and
/// emits a number of Domain [`Event`](Aggregate::Event)s.
#[async_trait]
pub trait Aggregate {
    /// Aggregate identifier: this should represent an unique identifier to
    /// refer to a unique Aggregate instance.
    type Id: Eq;

    /// State of the Aggregate: this should represent the Domain Entity data
    /// structure.
    type State: Default;

    /// Represents a specific, domain-related change to the Aggregate
    /// [`State`](Aggregate::State).
    type Event;

    /// Commands are all the possible operations available on an Aggregate.
    /// Use Commands to model business use-cases or [`State`](Aggregate::State)
    /// mutations.
    type Command;

    /// Possible failures while [`apply`](Aggregate::apply)ing
    /// [`Event`](Aggregate::Event)s or handling
    /// [`Command`](Aggregate::Command)s.
    type Error;

    /// Applies an [`Event`](Aggregate::Event) to the current Aggregate
    /// [`State`](Aggregate::State).
    ///
    /// To enforce immutability, this method takes ownership of the previous
    /// [`State`](Aggregate::State) and the current
    /// [`Event`](Aggregate::Event) to apply, and returns the new version of the
    /// [`State`](Aggregate::State) or an error.
    fn apply(state: Self::State, event: Self::Event) -> Result<Self::State, Self::Error>;

    /// Handles the requested [`Command`](Aggregate::Command) and returns a list
    /// of [`Event`](Aggregate::Event)s to apply the
    /// [`State`](Aggregate::State) mutation based on the current representation
    /// of the State.
    async fn handle(
        &self,
        id: &Self::Id,
        state: &Self::State,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error>;
}

/// Extension trait with some handy methods to use with [`Aggregate`]s.
pub trait AggregateExt: Aggregate {
    /// Applies a list of [`Event`](Aggregate::Event)s from an `Iterator`
    /// to the current Aggregate [`State`](Aggregate::State).
    ///
    /// Useful to recreate the [`State`](Aggregate::State) of an Aggregate when
    /// the [`Event`](Aggregate::Event)s are located in-memory.
    #[inline]
    fn fold<I>(state: Self::State, mut events: I) -> Result<Self::State, Self::Error>
    where
        I: Iterator<Item = Self::Event>,
    {
        events.try_fold(state, Self::apply)
    }
}

impl<T> AggregateExt for T where T: Aggregate {}

/// Factory type for new [`AggregateRoot`] instances.
#[derive(Clone)]
pub struct AggregateRootFactory<T>
where
    T: Aggregate,
{
    aggregate: T,
}

impl<T> From<T> for AggregateRootFactory<T>
where
    T: Aggregate,
{
    #[inline]
    fn from(aggregate: T) -> Self {
        Self { aggregate }
    }
}

impl<T> AggregateRootFactory<T>
where
    T: Aggregate + Clone,
{
    /// Builds a new [`AggregateRoot`] instance for the specified [`Aggregate`]
    /// [`Id`](Aggregate::Id).
    #[inline]
    pub fn build(&self, id: T::Id) -> AggregateRoot<T> {
        self.build_with_state(id, 0, Default::default())
    }

    /// Builds a new [`AggregateRoot`] instance for the specified Aggregate
    /// with a specified [`State`](Aggregate::State) value.
    #[inline]
    pub fn build_with_state(&self, id: T::Id, version: u32, state: T::State) -> AggregateRoot<T> {
        AggregateRoot {
            id,
            version,
            state,
            aggregate: self.aggregate.clone(),
            to_commit: Vec::default(),
        }
    }
}

/// An [`AggregateRoot`] represents an handler to the [`Aggregate`] it's
/// managing, such as:
///
/// * Owning its [`State`](Aggregate::State), [`Id`](Aggregate::Id) and version,
/// * Proxying [`Command`](Aggregate::Command)s to the [`Aggregate`] using the
///   current [`State`](Aggregate::State),
/// * Keeping a list of [`Event`](Aggregate::Event)s to commit after
///   [`Command`](Aggregate::Command) execution.
///
/// ## Initialize
///
/// An [`AggregateRoot`] can only be initialized using the
/// [`AggregateRootFactory`].
///
/// Check [`AggregateRootFactory::build`] for more information.

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct AggregateRoot<T>
where
    T: Aggregate + 'static,
{
    id: T::Id,
    version: u32,

    #[cfg_attr(feature = "serde", serde(flatten))]
    state: T::State,

    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    aggregate: T,

    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    to_commit: Vec<T::Event>,
}

impl<T> PartialEq for AggregateRoot<T>
where
    T: Aggregate,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> Versioned for AggregateRoot<T>
where
    T: Aggregate,
{
    #[inline]
    fn version(&self) -> u32 {
        self.version
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
{
    /// Returns a reference to the Aggregate [`Id`](Aggregate::Id) that
    /// represents the entity wrapped by this [`AggregateRoot`] instance.
    #[inline]
    pub fn id(&self) -> &T::Id {
        &self.id
    }

    /// Returns a reference to the current Aggregate
    /// [`State`](Aggregate::State).
    #[inline]
    pub fn state(&self) -> &T::State {
        &self.state
    }

    /// Takes the list of events to commit from the current instance,
    /// resetting it to `None`.
    #[inline]
    pub(crate) fn take_events_to_commit(&mut self) -> Vec<T::Event> {
        std::mem::take(&mut self.to_commit)
    }

    /// Returns a new [`AggregateRoot`] having the specified version.
    #[inline]
    pub(crate) fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
}

impl<T> Deref for AggregateRoot<T>
where
    T: Aggregate,
{
    type Target = T::State;

    fn deref(&self) -> &Self::Target {
        self.state()
    }
}

impl<T> AggregateRoot<T>
where
    T: Aggregate,
    T::Event: Clone,
    T::State: Clone,
    T::Command: Debug,
{
    /// Handles the submitted [`Command`](Aggregate::Command) using the
    /// [`Aggregate::handle`] method and updates the Aggregate
    /// [`State`](Aggregate::State).
    ///
    /// Returns a `&mut self` reference to allow for _method chaining_.
    #[cfg_attr(
        feature = "with-tracing",
        tracing::instrument(level = "debug", name = "AggregateRoot::handle", skip(self))
    )]
    pub async fn handle(&mut self, command: T::Command) -> Result<&mut Self, T::Error> {
        let mut events = self
            .aggregate
            .handle(self.id(), self.state(), command)
            .await?;

        // Only apply new events if the command handling actually
        // produced new ones.
        self.state = T::fold(self.state.clone(), events.iter().cloned())?;
        self.to_commit.append(&mut events);

        Ok(self)
    }
}
