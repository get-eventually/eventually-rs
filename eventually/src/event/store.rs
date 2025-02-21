//! Contains implementations of the [`event::Store`] trait and connected abstractions,
//! such as the [`std::collections::HashMap`]'s based [`InMemory`] Event Store implementation.

use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use futures::stream::{iter, StreamExt};

use crate::{event, message, version};

/// Interface used to stream [Persisted][event::Persisted] Domain Events
/// from an Event Store to an application.
pub trait Streamer<StreamId, Event>: Send + Sync
where
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
    /// The error type returned by the Store during a [`stream`] call.
    type Error: Send + Sync;

    /// Opens an Event Stream, effectively streaming all Domain Events
    /// of an Event Stream back in the application.
    fn stream(
        &self,
        id: &StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<StreamId, Event, Self::Error>;
}

/// All possible error types returned by [`Appender::append`].
#[derive(Debug, thiserror::Error)]
pub enum AppendError {
    /// Error returned when [`Appender::append`] encounters a conflict error
    /// while appending the new Domain Events.
    #[error("failed to append new domain events: {0}")]
    Conflict(#[from] version::ConflictError),
    /// Error returned when the [Appender] implementation has encountered an error.
    #[error("failed to append new domain events, an error occurred: {0}")]
    Internal(#[from] anyhow::Error),
}

#[async_trait]
/// Interface used to append new Domain Events in an Event Store.
pub trait Appender<StreamId, Event>: Send + Sync
where
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
    /// Appens new Domain Events to the specified Event Stream.
    ///
    /// The result of this operation is the new [Version][version::Version]
    /// of the Event Stream with the specified Domain Events added to it.
    async fn append(
        &self,
        id: StreamId,
        version_check: version::Check,
        events: Vec<event::Envelope<Event>>,
    ) -> Result<version::Version, AppendError>;
}

/// An [Event][event::Envelope] Store, used to store Domain Events in Event Streams -- a stream
/// of Domain Events -- and retrieve them.
///
/// Each Event Stream is represented by a unique Stream identifier.
pub trait Store<StreamId, Event>:
    Streamer<StreamId, Event> + Appender<StreamId, Event> + Send + Sync
where
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
}

impl<T, StreamId, Event> Store<StreamId, Event> for T
where
    T: Streamer<StreamId, Event> + Appender<StreamId, Event> + Send + Sync,
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
}

#[derive(Debug)]
struct InMemoryBackend<Id, Evt>
where
    Evt: message::Message,
{
    event_streams: HashMap<Id, Vec<event::Persisted<Id, Evt>>>,
}

impl<Id, Evt> Default for InMemoryBackend<Id, Evt>
where
    Evt: message::Message,
{
    fn default() -> Self {
        Self {
            event_streams: HashMap::default(),
        }
    }
}

/// In-memory implementation of [`event::Store`] trait,
/// backed by a thread-safe [`std::collections::HashMap`].
#[derive(Debug, Clone)]
pub struct InMemory<Id, Evt>
where
    Evt: message::Message,
{
    backend: Arc<RwLock<InMemoryBackend<Id, Evt>>>,
}

impl<Id, Evt> Default for InMemory<Id, Evt>
where
    Evt: message::Message,
{
    fn default() -> Self {
        Self {
            backend: Arc::default(),
        }
    }
}

impl<Id, Evt> Streamer<Id, Evt> for InMemory<Id, Evt>
where
    Id: Clone + Eq + Hash + Send + Sync,
    Evt: message::Message + Clone + Send + Sync,
{
    type Error = Infallible;

    fn stream(&self, id: &Id, select: event::VersionSelect) -> event::Stream<Id, Evt, Self::Error> {
        let backend = self
            .backend
            .read()
            .expect("acquire read lock on event store backend");

        let events = backend
            .event_streams
            .get(id)
            .cloned()
            .unwrap_or_default() // NOTE: the new Vec is empty, so there will be no memory allocation!
            .into_iter()
            .filter(move |evt| match select {
                event::VersionSelect::All => true,
                event::VersionSelect::From(v) => evt.version >= v,
            });

        iter(events).map(Ok).boxed()
    }
}

#[async_trait]
impl<Id, Evt> Appender<Id, Evt> for InMemory<Id, Evt>
where
    Id: Clone + Eq + Hash + Send + Sync,
    Evt: message::Message + Clone + Send + Sync,
{
    async fn append(
        &self,
        id: Id,
        version_check: version::Check,
        events: Vec<event::Envelope<Evt>>,
    ) -> Result<version::Version, AppendError> {
        let mut backend = self
            .backend
            .write()
            .expect("acquire write lock on event store backend");

        let last_event_stream_version = backend
            .event_streams
            .get(&id)
            .and_then(|events| events.last())
            .map(|event| event.version)
            .unwrap_or_default();

        if let version::Check::MustBe(expected) = version_check {
            if last_event_stream_version != expected {
                return Err(AppendError::Conflict(version::ConflictError {
                    expected,
                    actual: last_event_stream_version,
                }));
            }
        }

        let mut persisted_events: Vec<event::Persisted<Id, Evt>> = events
            .into_iter()
            .enumerate()
            .map(|(i, event)| event::Persisted {
                stream_id: id.clone(),
                version: last_event_stream_version + (i as u64) + 1,
                event,
            })
            .collect();

        let new_last_event_stream_version = persisted_events
            .last()
            .map(|evt| evt.version)
            .unwrap_or_default();

        backend
            .event_streams
            .entry(id)
            .and_modify(|events| events.append(&mut persisted_events))
            .or_insert_with(|| persisted_events);

        Ok(new_last_event_stream_version)
    }
}

/// Decorator type for an [`event::Store`] implementation that tracks the list of
/// recorded Domain Events through it.
///
/// Useful for testing purposes, i.e. asserting that Domain Events written throguh
/// this Event Store instance are the ones expected.
#[derive(Debug, Clone)]
pub struct Tracking<T, StreamId, Event>
where
    T: Store<StreamId, Event> + Send + Sync,
    StreamId: Send + Sync,
    Event: message::Message + Send + Sync,
{
    store: T,

    #[allow(clippy::type_complexity)] // It is a complex type but still readable.
    events: Arc<RwLock<Vec<event::Persisted<StreamId, Event>>>>,
}

impl<T, StreamId, Event> Tracking<T, StreamId, Event>
where
    T: Store<StreamId, Event> + Send + Sync,
    StreamId: Clone + Send + Sync,
    Event: message::Message + Clone + Send + Sync,
{
    /// Returns the list of recoded Domain Events through this decorator so far.
    ///
    /// # Panics
    ///
    /// Since the internal data is thread-safe through an [`RwLock`], this method
    /// could potentially panic while attempting to get a read-only lock on the data recorded.
    pub fn recorded_events(&self) -> Vec<event::Persisted<StreamId, Event>> {
        self.events
            .read()
            .expect("acquire lock on recorded events list")
            .clone()
    }

    /// Resets the list of recorded Domain Events through this decorator.
    ///
    /// # Panics
    ///
    /// Since the internal data is thread-safe through an [`RwLock`], this method
    /// could potentially panic while attempting to get a read-write lock to empty the internal store.
    pub fn reset_recorded_events(&self) {
        self.events
            .write()
            .expect("acquire lock on recorded events list")
            .clear();
    }
}

impl<T, StreamId, Event> Streamer<StreamId, Event> for Tracking<T, StreamId, Event>
where
    T: Store<StreamId, Event> + Send + Sync,
    StreamId: Clone + Send + Sync,
    Event: message::Message + Clone + Send + Sync,
{
    type Error = <T as Streamer<StreamId, Event>>::Error;

    fn stream(
        &self,
        id: &StreamId,
        select: event::VersionSelect,
    ) -> event::Stream<StreamId, Event, Self::Error> {
        self.store.stream(id, select)
    }
}

#[async_trait]
impl<T, StreamId, Event> Appender<StreamId, Event> for Tracking<T, StreamId, Event>
where
    T: Store<StreamId, Event> + Send + Sync,
    StreamId: Clone + Send + Sync,
    Event: message::Message + Clone + Send + Sync,
{
    async fn append(
        &self,
        id: StreamId,
        version_check: version::Check,
        events: Vec<event::Envelope<Event>>,
    ) -> Result<version::Version, AppendError> {
        let new_version = self
            .store
            .append(id.clone(), version_check, events.clone())
            .await?;

        let events_size = events.len();
        let previous_version = new_version - (events_size as version::Version);

        let mut persisted_events = events
            .into_iter()
            .enumerate()
            .map(|(i, event)| event::Persisted {
                stream_id: id.clone(),
                version: previous_version + (i as version::Version) + 1,
                event,
            })
            .collect();

        self.events
            .write()
            .expect("acquire lock on recorded events list")
            .append(&mut persisted_events);

        Ok(new_version)
    }
}

/// Extension trait that can be used to pull in supertypes implemented
/// in this module.
pub trait EventStoreExt<StreamId, Event>: Store<StreamId, Event> + Send + Sync + Sized
where
    StreamId: Clone + Send + Sync,
    Event: message::Message + Clone + Send + Sync,
{
    /// Returns a [`Tracking`] instance that decorates the original [`event::Store`]
    /// instanca this method has been called on.
    fn with_recorded_events_tracking(self) -> Tracking<Self, StreamId, Event> {
        Tracking {
            store: self,
            events: Arc::default(),
        }
    }
}

impl<T, StreamId, Event> EventStoreExt<StreamId, Event> for T
where
    T: Store<StreamId, Event> + Send + Sync,
    StreamId: Clone + Send + Sync,
    Event: message::Message + Clone + Send + Sync,
{
}

#[allow(clippy::semicolon_if_nothing_returned)] // False positives :shrugs:
#[cfg(test)]
mod test {
    use futures::TryStreamExt;
    use lazy_static::lazy_static;

    use super::*;
    use crate::event;
    use crate::event::store::{Appender, Streamer};
    use crate::message::tests::StringMessage;
    use crate::version::Version;

    const STREAM_ID: &str = "stream:test";

    lazy_static! {
        static ref EVENTS: Vec<event::Envelope<StringMessage>> = vec![
            event::Envelope::from(StringMessage("event-1")),
            event::Envelope::from(StringMessage("event-2")),
            event::Envelope::from(StringMessage("event-3")),
        ];
    }

    #[tokio::test]
    async fn it_works() {
        let event_store = InMemory::<&'static str, StringMessage>::default();

        let new_event_stream_version = event_store
            .append(STREAM_ID, version::Check::MustBe(0), EVENTS.clone())
            .await
            .expect("append should not fail");

        let expected_version = EVENTS.len() as Version;
        assert_eq!(expected_version, new_event_stream_version);

        let expected_events = EVENTS
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, event)| event::Persisted {
                stream_id: STREAM_ID,
                version: (i as Version) + 1,
                event,
            })
            .collect::<Vec<_>>();

        let event_stream: Vec<_> = event_store
            .stream(&STREAM_ID, event::VersionSelect::All)
            .try_collect()
            .await
            .expect("opening an event stream should not fail");

        assert_eq!(expected_events, event_stream);
    }

    #[tokio::test]
    async fn tracking_store_works() {
        let event_store = InMemory::<&'static str, StringMessage>::default();
        let tracking_event_store = event_store.with_recorded_events_tracking();

        tracking_event_store
            .append(STREAM_ID, version::Check::MustBe(0), EVENTS.clone())
            .await
            .expect("append should not fail");

        let event_stream: Vec<_> = tracking_event_store
            .stream(&STREAM_ID, event::VersionSelect::All)
            .try_collect()
            .await
            .expect("opening an event stream should not fail");

        assert_eq!(event_stream, tracking_event_store.recorded_events());
    }

    #[tokio::test]
    async fn version_conflict_checks_work_as_expected() {
        let event_store = InMemory::<&'static str, StringMessage>::default();

        let append_error = event_store
            .append(STREAM_ID, version::Check::MustBe(3), EVENTS.clone())
            .await
            .expect_err("the event stream version should be zero");

        if let AppendError::Conflict(err) = append_error {
            return assert_eq!(
                version::ConflictError {
                    expected: 3,
                    actual: 0,
                },
                err
            );
        }

        panic!("expected conflict error, received: {append_error}")
    }
}
