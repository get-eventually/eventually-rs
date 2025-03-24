use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{SystemTime, UNIX_EPOCH};

use eventually::aggregate;
use eventually::aggregate::Aggregate;
use eventually::message::Message;
use eventually_macros::aggregate_root;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestAggregateId(pub i64);

impl Display for TestAggregateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "test-aggregate:{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestDomainEvent {
    WasCreated {
        id: TestAggregateId,
        name: String,
        at: u128,
    },
    WasDeleted {
        id: TestAggregateId,
    },
}

impl Message for TestDomainEvent {
    fn name(&self) -> &'static str {
        match self {
            TestDomainEvent::WasCreated { .. } => "TestDomainSomethingWasCreated",
            TestDomainEvent::WasDeleted { .. } => "TestDomainSomethingWasDeleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestAggregate {
    id: TestAggregateId,
    name: String,
    is_deleted: bool,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TestAggregateError {
    #[error("already exists")]
    AlreadyExists,
    #[error("not created yet")]
    NotCreatedYet,
}

impl Aggregate for TestAggregate {
    type Id = TestAggregateId;
    type Event = TestDomainEvent;
    type Error = TestAggregateError;

    fn type_name() -> &'static str {
        "TestAggregate"
    }

    fn aggregate_id(&self) -> &Self::Id {
        &self.id
    }

    fn apply(state: Option<Self>, event: Self::Event) -> Result<Self, Self::Error> {
        match (state, event) {
            (None, TestDomainEvent::WasCreated { id, name, .. }) => Ok(Self {
                id,
                name,
                is_deleted: false,
            }),
            (Some(_), TestDomainEvent::WasCreated { .. }) => Err(TestAggregateError::AlreadyExists),
            (Some(mut a), TestDomainEvent::WasDeleted { .. }) => {
                a.is_deleted = true;
                Ok(a)
            },
            (None, TestDomainEvent::WasDeleted { .. }) => Err(TestAggregateError::NotCreatedYet),
        }
    }
}

#[aggregate_root(TestAggregate)]
#[derive(Debug, Clone, PartialEq)]
pub struct TestAggregateRoot;

impl TestAggregateRoot {
    pub fn create(id: TestAggregateId, name: String) -> Result<Self, TestAggregateError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        Ok(aggregate::Root::<TestAggregate>::record_new(
            TestDomainEvent::WasCreated { name, id, at: now }.into(),
        )?
        .into())
    }

    pub fn delete(&mut self) -> Result<(), TestAggregateError> {
        let id = self.id;

        if !self.is_deleted {
            self.record_that(TestDomainEvent::WasDeleted { id }.into())?;
        }

        Ok(())
    }
}
