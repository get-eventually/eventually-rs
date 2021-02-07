use std::fmt::Debug;

use futures::stream::TryStreamExt;

use crate::command::CommandHandler;
use crate::eventstore::{
    EventStore, PersistedEvent, Select, StreamInstance, StreamName, VersionCheck,
};
use crate::inmemory::InMemoryEventStore;

pub struct CommandHandlerScenario;

impl CommandHandlerScenario {
    pub fn given<E>(events: Vec<PersistedEvent<E>>) -> CommandHandlerGiven<E> {
        CommandHandlerGiven(events)
    }

    pub fn when<E, C>(command: C) -> CommandHandlerWhen<E, C> {
        CommandHandlerWhen {
            given: Vec::new(),
            when: command,
        }
    }
}

pub struct CommandHandlerGiven<E>(Vec<PersistedEvent<E>>);

impl<E> CommandHandlerGiven<E> {
    pub fn when<C>(self, command: C) -> CommandHandlerWhen<E, C> {
        CommandHandlerWhen {
            given: self.0,
            when: command,
        }
    }
}

pub struct CommandHandlerWhen<T, C> {
    given: Vec<PersistedEvent<T>>,
    when: C,
}

impl<T, C> CommandHandlerWhen<T, C> {
    pub fn then<E>(self, events: Vec<PersistedEvent<T>>) -> CommandHandlerThen<T, C, E> {
        CommandHandlerThen {
            given: self.given,
            when: self.when,
            then: Ok(events),
        }
    }

    pub fn then_error<E>(self, err: E) -> CommandHandlerThen<T, C, E> {
        CommandHandlerThen {
            given: self.given,
            when: self.when,
            then: Err(err),
        }
    }
}

pub struct CommandHandlerThen<T, C, E> {
    given: Vec<PersistedEvent<T>>,
    when: C,
    then: Result<Vec<PersistedEvent<T>>, E>,
}

impl<T, C, E> CommandHandlerThen<T, C, E>
where
    T: Debug + PartialEq + Clone + Send + Sync,
    E: Debug + PartialEq,
{
    pub async fn assert_on<CH, F>(self, handler_factory: F)
    where
        CH: CommandHandler<C, Error = E>,
        CH::Error: Debug,
        F: FnOnce(InMemoryEventStore<T>) -> CH,
    {
        let mut event_store = InMemoryEventStore::<T>::default();
        let expected_offset = self.given.len();

        for event in self.given {
            let stream_name = StreamInstance(&event.stream_type, &event.stream_name);

            event_store
                .append(stream_name, VersionCheck::Any, vec![event.event])
                .await
                .unwrap();
        }

        let mut command_handler = handler_factory(event_store.clone());
        let result = command_handler.handle(self.when).await;

        if self.then.is_err() {
            assert_eq!(self.then.unwrap_err(), result.unwrap_err());
            return;
        }

        result.unwrap();

        let recorded_events: Vec<PersistedEvent<T>> = event_store
            .stream(StreamName::All, Select::From(expected_offset as u64))
            .try_collect()
            .await
            .unwrap();

        assert_eq!(self.then.unwrap(), recorded_events);
    }
}
