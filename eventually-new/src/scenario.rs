use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::aggregate::{Aggregate, AggregateRootBuilder, AggregateRootError, Repository};
use crate::eventstore::{EventStore, Version};
use crate::inmemory::InMemoryEventStore;
use crate::Events;

pub struct AggregateRootScenario<A>
where
    A: Aggregate,
{
    aggregate_id: A::Id,
    aggregate_root_builder: AggregateRootBuilder<A>,
}

impl<A> AggregateRootScenario<A>
where
    A: Aggregate,
{
    pub fn with(id: A::Id, aggregate: A) -> Self {
        Self {
            aggregate_id: id,
            aggregate_root_builder: AggregateRootBuilder::from(aggregate),
        }
    }

    pub fn given(self, events: Events<A::DomainEvent>) -> AggregateRootScenarioWhen<A> {
        AggregateRootScenarioWhen {
            aggregate_id: self.aggregate_id,
            aggregate_root_builder: self.aggregate_root_builder,
            given: Some(events),
        }
    }

    pub fn when(self, command: A::Command) -> AggregateRootScenarioThen<A> {
        AggregateRootScenarioThen {
            aggregate_id: self.aggregate_id,
            aggregate_root_builder: self.aggregate_root_builder,
            given: None,
            when: command,
        }
    }
}

pub struct AggregateRootScenarioWhen<A>
where
    A: Aggregate,
{
    aggregate_id: A::Id,
    aggregate_root_builder: AggregateRootBuilder<A>,
    given: Option<Events<A::DomainEvent>>,
}

impl<A> AggregateRootScenarioWhen<A>
where
    A: Aggregate,
{
    pub fn when(self, command: A::Command) -> AggregateRootScenarioThen<A> {
        AggregateRootScenarioThen {
            aggregate_id: self.aggregate_id,
            aggregate_root_builder: self.aggregate_root_builder,
            given: self.given,
            when: command,
        }
    }
}

pub struct AggregateRootScenarioThen<A>
where
    A: Aggregate,
{
    aggregate_id: A::Id,
    aggregate_root_builder: AggregateRootBuilder<A>,
    given: Option<Events<A::DomainEvent>>,
    when: A::Command,
}

impl<A> AggregateRootScenarioThen<A>
where
    A: Aggregate + Clone + 'static,
    <A as Aggregate>::Id: Eq + Hash + Display + Debug + Clone + Unpin,
    <A as Aggregate>::State: Default + Clone,
    <A as Aggregate>::DomainEvent: PartialEq + Debug + Clone + Unpin,
    <A as Aggregate>::ApplyError: PartialEq + StdError + 'static,
    <A as Aggregate>::Command: AsRef<A::Id> + Debug,
    <A as Aggregate>::HandleError: PartialEq + StdError + 'static,
{
    pub async fn then_nothing(self) {
        self.then(Vec::new()).await
    }

    pub async fn then(self, expect: Events<A::DomainEvent>) {
        let mut event_store = InMemoryEventStore::<A::Id, A::DomainEvent>::default();

        if let Some(events) = self.given {
            event_store
                .append(&self.aggregate_id, Version::Any, events)
                .await
                .expect("given events should be in the event store");
        }

        let repository = Repository::new(self.aggregate_root_builder, event_store);

        let mut root = repository
            .get(&self.aggregate_id)
            .await
            .expect("aggregate root should be found");

        root.handle(self.when)
            .await
            .expect("command handling should be successful");

        assert_eq!(expect, root.flush_events());
    }

    pub async fn then_error(self, error: A::HandleError) {
        let mut event_store = InMemoryEventStore::<A::Id, A::DomainEvent>::default();

        if let Some(events) = self.given {
            event_store
                .append(&self.aggregate_id, Version::Any, events)
                .await
                .expect("given events should be in the event store");
        }

        let repository = Repository::new(self.aggregate_root_builder, event_store);

        let mut root = repository
            .get(&self.aggregate_id)
            .await
            .expect("aggregate root should be found");

        let err = root
            .handle(self.when)
            .await
            .expect_err("command should fail");

        assert_eq!(AggregateRootError::Handle(error), err);
    }
}
