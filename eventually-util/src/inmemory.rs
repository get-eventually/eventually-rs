//! Contains supporting entities using an in-memory backend.

use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use eventually_core::aggregate::Aggregate;
use eventually_core::store::persistent::EventBuilderWithVersion;
use eventually_core::store::{AppendError, EventStream, Expected, PersistedEvent, Select};
use eventually_core::versioning::Versioned;

use futures::future::BoxFuture;
use futures::stream::{empty, iter, StreamExt};

use parking_lot::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "inmemory::EventStore: conflicting versions, expected {expected}, got instead {actual}"
    )]
    Conflict { expected: u32, actual: u32 },
}

impl AppendError for Error {
    fn is_conflict_error(&self) -> bool {
        true
    }
}

/// Builder for [`EventStore`] instances.
///
/// [`EventStore`]: struct.EventStore.html
pub struct EventStoreBuilder;

impl EventStoreBuilder {
    /// Builds a new [`EventStore`] instance compatible with the provided [`Aggregate`].
    ///
    /// [`Aggregate`]: ../../eventually-core/aggregate/trait.Aggregate.html
    #[inline]
    pub fn for_aggregate<T>(_: &T) -> EventStore<T::Id, T::Event>
    where
        T: Aggregate,
        T::Id: Hash + Eq,
    {
        Default::default()
    }
}

/// An in-memory [`EventStore`] implementation, backed by an [`HashMap`].
///
/// [`EventStore`]: ../../eventually_core/store/trait.EventStore.html
/// [`HashMap`]: something
#[derive(Debug, Clone)]
pub struct EventStore<Id, Event>
where
    Id: Hash + Eq,
{
    global_offset: Arc<AtomicU32>,
    backend: Arc<RwLock<HashMap<Id, Vec<PersistedEvent<Event>>>>>,
}

impl<Id, Event> Default for EventStore<Id, Event>
where
    Id: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Self {
            global_offset: Arc::new(AtomicU32::new(0)),
            backend: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<Id, Event> eventually_core::store::EventStore for EventStore<Id, Event>
where
    Id: Hash + Eq + Sync + Send + Clone,
    Event: Sync + Send + Clone,
{
    type SourceId = Id;
    type Event = Event;
    type Error = Error;

    fn append(
        &mut self,
        id: Self::SourceId,
        version: Expected,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<u32, Self::Error>> {
        Box::pin(async move {
            let mut last_version = self
                .backend
                .read()
                .get(&id)
                .and_then(|events| events.last())
                .map(|event| event.version())
                .unwrap_or(0);

            if let Expected::Exact(version) = version {
                if version != last_version {
                    return Err(Error::Conflict {
                        expected: version,
                        actual: last_version,
                    });
                }
            }

            let mut persisted_events = into_persisted_events(last_version, events)
                .into_iter()
                .map(|event| {
                    let offset = self.global_offset.fetch_add(1, Ordering::SeqCst);
                    event.sequence_number(offset)
                })
                .collect::<Vec<PersistedEvent<Event>>>();

            last_version = persisted_events
                .last()
                .map(|event| event.version())
                .unwrap_or(last_version);

            self.backend
                .write()
                .entry(id)
                .and_modify(|events| events.append(&mut persisted_events))
                .or_insert_with(|| persisted_events);

            Ok(last_version)
        })
    }

    fn stream(
        &self,
        id: Self::SourceId,
        select: Select,
    ) -> BoxFuture<Result<EventStream<Self>, Self::Error>> {
        Box::pin(async move {
            Ok(self
                .backend
                .read()
                .get(&id)
                .map(move |events| {
                    let stream = events
                        .clone()
                        .into_iter()
                        .filter(move |event| match select {
                            Select::All => true,
                            Select::From(v) => event.version() >= v,
                        });

                    iter(stream).map(Ok).boxed()
                })
                .unwrap_or_else(|| empty().boxed()))
        })
    }

    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            self.backend.write().remove(&id);

            Ok(())
        })
    }
}

fn into_persisted_events<T>(last_version: u32, events: Vec<T>) -> Vec<EventBuilderWithVersion<T>> {
    events
        .into_iter()
        .enumerate()
        .map(|(i, event)| PersistedEvent::from(event).version(last_version + (i as u32) + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::EventStore as InMemoryStore;
    use eventually_core::store::{EventStore, Expected, PersistedEvent, Select};

    use futures::StreamExt;

    use tokio_test::block_on;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum Event {
        A,
        B,
        C,
    }

    #[test]
    fn append() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        append_to(&mut store, "test-append");
    }

    #[test]
    fn remove() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        append_to(&mut store, "test-remove");

        assert!(block_on(store.remove("test-remove")).is_ok());
        assert!(block_on(stream_to_vec_from(&store, "test-remove", 0)).is_empty());
    }

    #[test]
    fn stream() {
        let mut store = InMemoryStore::<&'static str, Event>::default();

        let events_1 = vec![Event::A, Event::B, Event::C];
        let events_2 = vec![Event::B, Event::A];

        assert!(block_on(store.append("test-stream", Expected::Any, events_1.clone())).is_ok());
        assert!(
            block_on(store.append("test-stream", Expected::Exact(3), events_2.clone())).is_ok()
        );

        assert_eq!(
            block_on(stream_to_vec_from(&store, "test-stream", 1)),
            vec![
                PersistedEvent::from(Event::A).version(1).sequence_number(0),
                PersistedEvent::from(Event::B).version(2).sequence_number(1),
                PersistedEvent::from(Event::C).version(3).sequence_number(2),
                PersistedEvent::from(Event::B).version(4).sequence_number(3),
                PersistedEvent::from(Event::A).version(5).sequence_number(4)
            ]
        );

        assert_eq!(
            block_on(stream_to_vec_from(&store, "test-stream", 4)),
            vec![
                PersistedEvent::from(Event::B).version(4).sequence_number(3),
                PersistedEvent::from(Event::A).version(5).sequence_number(4)
            ]
        );
    }

    fn append_to(store: &mut InMemoryStore<&'static str, Event>, id: &'static str) {
        let events = vec![Event::A, Event::B, Event::C];

        assert!(block_on(store.append(id, Expected::Exact(0), events.clone())).is_ok());

        assert_eq!(
            block_on(stream_to_vec_from(&store, id, 0)),
            vec![
                PersistedEvent::from(Event::A).version(1).sequence_number(0),
                PersistedEvent::from(Event::B).version(2).sequence_number(1),
                PersistedEvent::from(Event::C).version(3).sequence_number(2)
            ]
        );
    }

    async fn stream_to_vec_from(
        store: &InMemoryStore<&'static str, Event>,
        id: &'static str,
        from: u32,
    ) -> Vec<PersistedEvent<Event>> {
        store
            .stream(id, Select::From(from))
            .await
            .expect("should be infallible")
            .map(|event| event.expect("should be infallible"))
            .collect()
            .await
    }
}
