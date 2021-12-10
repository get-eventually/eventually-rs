use std::{fmt::Debug, hash::Hash};

use crate::{command, command::Command, event, event::Store, test, test::store::EventStoreExt};

pub struct Scenario;

impl Scenario {
    pub fn given<Id, Evt>(self, events: Vec<event::Persisted<Id, Evt>>) -> ScenarioGiven<Id, Evt> {
        ScenarioGiven { given: events }
    }

    pub fn when<Id, Evt, Cmd>(self, command: Command<Cmd>) -> ScenarioWhen<Id, Evt, Cmd> {
        ScenarioWhen {
            given: Vec::default(),
            when: command,
        }
    }
}

pub struct ScenarioGiven<Id, Evt> {
    given: Vec<event::Persisted<Id, Evt>>,
}

impl<Id, Evt> ScenarioGiven<Id, Evt> {
    pub fn when<Cmd>(self, command: Command<Cmd>) -> ScenarioWhen<Id, Evt, Cmd> {
        ScenarioWhen {
            given: self.given,
            when: command,
        }
    }
}

pub struct ScenarioWhen<Id, Evt, Cmd> {
    given: Vec<event::Persisted<Id, Evt>>,
    when: Command<Cmd>,
}

impl<Id, Evt, Cmd> ScenarioWhen<Id, Evt, Cmd> {
    pub fn then(self, events: Vec<event::Persisted<Id, Evt>>) -> ScenarioThen<Id, Evt, Cmd> {
        ScenarioThen {
            given: self.given,
            when: self.when,
            case: ScenarioThenCase::Produces(events),
        }
    }

    pub fn then_fails(self) -> ScenarioThen<Id, Evt, Cmd> {
        ScenarioThen {
            given: self.given,
            when: self.when,
            case: ScenarioThenCase::Fails,
        }
    }
}

enum ScenarioThenCase<Id, Evt> {
    Produces(Vec<event::Persisted<Id, Evt>>),
    Fails,
}

pub struct ScenarioThen<Id, Evt, Cmd> {
    given: Vec<event::Persisted<Id, Evt>>,
    when: Command<Cmd>,
    case: ScenarioThenCase<Id, Evt>,
}

impl<Id, Evt, Cmd> ScenarioThen<Id, Evt, Cmd>
where
    Id: Clone + Eq + Hash + Send + Sync + Debug,
    Evt: Clone + PartialEq + Send + Sync + Debug,
{
    pub async fn assert_on<F, H>(self, handler_factory: F)
    where
        F: Fn(test::store::Tracking<test::store::InMemory<Id, Evt>>) -> H,
        H: command::Handler<Cmd>,
    {
        let event_store = test::store::InMemory::<Id, Evt>::default();
        let tracking_event_store = event_store.clone().with_recorded_events_tracking();

        for event in self.given {
            event_store
                .append(
                    event.stream_id,
                    event::StreamVersionExpected::MustBe(event.version - 1),
                    vec![event.payload],
                )
                .await
                .expect("domain event in 'given' should be inserted in the event store");
        }

        let handler = handler_factory(tracking_event_store.clone());
        let result = handler.handle(self.when).await;

        match self.case {
            ScenarioThenCase::Produces(events) => {
                let recorded_events = tracking_event_store.recorded_events();
                assert_eq!(events, recorded_events);
            }
            ScenarioThenCase::Fails => assert!(result.is_err()),
        };
    }
}
