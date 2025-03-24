//! Module exposing a test [Scenario] type to write Domain [Command][command::Envelope]s
//! test cases using the [given-then-when canvas](https://www.agilealliance.org/glossary/gwt/).

use std::fmt::Debug;
use std::hash::Hash;

use crate::event::store::{Appender, EventStoreExt};
use crate::{command, event, message, version};

/// A test scenario that can be used to test a [Command][command::Envelope] [Handler][command::Handler]
/// using a [given-then-when canvas](https://www.agilealliance.org/glossary/gwt/) approach.
pub struct Scenario;

impl Scenario {
    /// Sets the precondition state of the system for the [Scenario], which
    /// is expressed by a list of Domain [Event][event::Envelope]s in an Event-sourced system.
    #[must_use]
    pub fn given<Id, Evt>(self, events: Vec<event::Persisted<Id, Evt>>) -> ScenarioGiven<Id, Evt>
    where
        Evt: message::Message,
    {
        ScenarioGiven { given: events }
    }

    /// Specifies the [Command][command::Envelope] to test in the [Scenario], in the peculiar case
    /// of having a clean system.
    ///
    /// This is a shortcut for:
    /// ```text
    /// Scenario::given(vec![]).when(...)
    /// ```
    #[must_use]
    pub fn when<Id, Evt, Cmd>(self, command: command::Envelope<Cmd>) -> ScenarioWhen<Id, Evt, Cmd>
    where
        Evt: message::Message,
        Cmd: message::Message,
    {
        ScenarioWhen {
            given: Vec::default(),
            when: command,
        }
    }
}

#[doc(hidden)]
pub struct ScenarioGiven<Id, Evt>
where
    Evt: message::Message,
{
    given: Vec<event::Persisted<Id, Evt>>,
}

impl<Id, Evt> ScenarioGiven<Id, Evt>
where
    Evt: message::Message,
{
    /// Specifies the [Command][command::Envelope] to test in the [Scenario].
    #[must_use]
    pub fn when<Cmd>(self, command: command::Envelope<Cmd>) -> ScenarioWhen<Id, Evt, Cmd>
    where
        Cmd: message::Message,
    {
        ScenarioWhen {
            given: self.given,
            when: command,
        }
    }
}

#[doc(hidden)]
pub struct ScenarioWhen<Id, Evt, Cmd>
where
    Evt: message::Message,
    Cmd: message::Message,
{
    given: Vec<event::Persisted<Id, Evt>>,
    when: command::Envelope<Cmd>,
}

impl<Id, Evt, Cmd> ScenarioWhen<Id, Evt, Cmd>
where
    Evt: message::Message,
    Cmd: message::Message,
{
    /// Sets the expectation on the result of the [Scenario] to be positive
    /// and produce a specified list of Domain [Event]s.
    #[must_use]
    pub fn then(self, events: Vec<event::Persisted<Id, Evt>>) -> ScenarioThen<Id, Evt, Cmd> {
        ScenarioThen {
            given: self.given,
            when: self.when,
            case: ScenarioThenCase::Produces(events),
        }
    }

    /// Sets the expectation on the result of the [Scenario] to return an error.
    #[must_use]
    pub fn then_fails(self) -> ScenarioThen<Id, Evt, Cmd> {
        ScenarioThen {
            given: self.given,
            when: self.when,
            case: ScenarioThenCase::Fails,
        }
    }
}

enum ScenarioThenCase<Id, Evt>
where
    Evt: message::Message,
{
    Produces(Vec<event::Persisted<Id, Evt>>),
    Fails,
}

#[doc(hidden)]
pub struct ScenarioThen<Id, Evt, Cmd>
where
    Evt: message::Message,
    Cmd: message::Message,
{
    given: Vec<event::Persisted<Id, Evt>>,
    when: command::Envelope<Cmd>,
    case: ScenarioThenCase<Id, Evt>,
}

impl<Id, Evt, Cmd> ScenarioThen<Id, Evt, Cmd>
where
    Id: Clone + Eq + Hash + Send + Sync + Debug,
    Evt: message::Message + Clone + PartialEq + Send + Sync + Debug,
    Cmd: message::Message,
{
    /// Executes the whole [Scenario] by constructing a Command [Handler][command::Handler]
    /// with the provided closure function and running the specified assertions.
    ///
    /// # Panics
    ///
    /// The method panics if the assertion fails.
    pub async fn assert_on<F, H>(self, handler_factory: F)
    where
        F: Fn(event::store::Tracking<event::store::InMemory<Id, Evt>, Id, Evt>) -> H,
        H: command::Handler<Cmd>,
    {
        let event_store = event::store::InMemory::<Id, Evt>::default();
        let tracking_event_store = event_store.clone().with_recorded_events_tracking();

        for event in self.given {
            event_store
                .append(
                    event.stream_id,
                    version::Check::MustBe(event.version - 1),
                    vec![event.event],
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
            },
            ScenarioThenCase::Fails => assert!(result.is_err()),
        }
    }
}
