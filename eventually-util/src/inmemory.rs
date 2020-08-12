//! Contains supporting entities using an in-memory backend.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use eventually_core::aggregate::Aggregate;
use eventually_core::store::persistent::EventBuilderWithVersion;
use eventually_core::store::{AppendError, EventStream, Expected, Persisted, Select};
use eventually_core::versioning::Versioned;

use futures::future::BoxFuture;
use futures::stream::{empty, iter, StreamExt};

use parking_lot::RwLock;

/// Error returned by the [`EventStore::append`] when a conflict has been detected.
///
/// [`EventStore::append`]: trait.EventStore.html#method.append
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// Version conflict registered.
    #[error(
        "inmemory::EventStore: conflicting versions, expected {expected}, got instead {actual}"
    )]
    Conflict {
        /// The last version value found the Store.
        expected: u32,
        /// The actual version passed by the caller to the Store.
        actual: u32,
    },
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
    backend: Arc<RwLock<HashMap<Id, Vec<Persisted<Id, Event>>>>>,
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
            let expected = self
                .backend
                .read()
                .get(&id)
                .and_then(|events| events.last())
                .map(|event| event.version())
                .unwrap_or(0);

            if let Expected::Exact(actual) = version {
                if expected != actual {
                    return Err(Error::Conflict { expected, actual });
                }
            }

            let mut persisted_events: Vec<Persisted<Id, Event>> =
                into_persisted_events(expected, id.clone(), events)
                    .into_iter()
                    .map(|event| {
                        let offset = self.global_offset.fetch_add(1, Ordering::SeqCst);
                        event.sequence_number(offset)
                    })
                    .collect();

            let last_version = persisted_events
                .last()
                .map(Persisted::version)
                .unwrap_or(expected);

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

    fn stream_all(&self, select: Select) -> BoxFuture<Result<EventStream<Self>, Self::Error>> {
        let mut events: Vec<Persisted<Id, Event>> = self
            .backend
            .read()
            .values()
            .flatten()
            .cloned()
            .filter(move |event| match select {
                Select::All => true,
                Select::From(sequence_number) => event.sequence_number() >= sequence_number,
            })
            .collect();

        // Events must be sorted by the sequence number when using $all.
        events.sort_by(|a, b| a.sequence_number().cmp(&b.sequence_number()));

        Box::pin(futures::future::ok(iter(events).map(Ok).boxed()))
    }

    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            self.backend.write().remove(&id);

            Ok(())
        })
    }
}

fn into_persisted_events<Id, T>(
    last_version: u32,
    id: Id,
    events: Vec<T>,
) -> Vec<EventBuilderWithVersion<Id, T>>
where
    Id: Clone,
{
    events
        .into_iter()
        .enumerate()
        .map(|(i, event)| Persisted::from(id.clone(), event).version(last_version + (i as u32) + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Error, EventStore as InMemoryStore};
    use eventually_core::store::{EventStore, Expected, Persisted, Select};

    use futures::TryStreamExt;

    use tokio_test::block_on;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum Event {
        A,
        B,
        C,
    }

    #[test]
    fn append_with_any_versions_works() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        let id = "test-append";

        let events = vec![Event::A, Event::B, Event::C];

        let result = block_on(store.append(id, Expected::Any, events.clone()));
        assert!(result.is_ok());

        let result = block_on(store.append(id, Expected::Any, events.clone()));
        assert!(result.is_ok());
    }

    #[test]
    fn append_with_expected_versions_works() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        let id = "test-append";

        let events = vec![Event::A, Event::B, Event::C];

        let result = block_on(store.append(id, Expected::Exact(0), events.clone()));
        assert!(result.is_ok());

        let last_version = result.unwrap();
        let result = block_on(store.append(id, Expected::Exact(last_version), events.clone()));
        assert!(result.is_ok());
    }

    #[test]
    fn append_with_wrong_expected_versions_fails() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        let id = "test-append";

        let events = vec![Event::A, Event::B, Event::C];

        let result = block_on(store.append(id, Expected::Exact(0), events.clone()));
        assert!(result.is_ok());

        let last_version = result.unwrap();
        let poisoned_last_version = last_version + 1; // Poison the last version on purpose

        let result =
            block_on(store.append(id, Expected::Exact(poisoned_last_version), events.clone()));

        assert_eq!(
            Err(Error::Conflict {
                expected: last_version,
                actual: poisoned_last_version
            }),
            result
        );
    }

    #[test]
    fn remove() {
        let id = "test-remove";
        let mut store = InMemoryStore::<&'static str, Event>::default();

        // Removing an empty stream works.
        assert!(block_on(store.remove(id)).is_ok());
        assert!(block_on(stream_to_vec(&store, id, Select::All))
            .unwrap()
            .is_empty());

        // Add some events and lets remove them after
        let events = vec![Event::A, Event::B, Event::C];
        let result = block_on(store.append(id, Expected::Exact(0), events.clone()));
        assert!(result.is_ok());

        assert!(block_on(store.remove(id)).is_ok());
        assert!(block_on(stream_to_vec(&store, id, Select::All))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn stream() {
        let id = "test-stream";
        let mut store = InMemoryStore::<&'static str, Event>::default();

        let events_1 = vec![Event::A, Event::B, Event::C];
        let events_2 = vec![Event::B, Event::A];

        let result = block_on(store.append(id, Expected::Exact(0), events_1));
        assert!(result.is_ok());

        let last_version = result.unwrap();
        let result = block_on(store.append(id, Expected::Exact(last_version), events_2));
        assert!(result.is_ok());

        // Stream from the start.
        assert_eq!(
            block_on(stream_to_vec(&store, id, Select::All)).unwrap(),
            vec![
                Persisted::from(id, Event::A).version(1).sequence_number(0),
                Persisted::from(id, Event::B).version(2).sequence_number(1),
                Persisted::from(id, Event::C).version(3).sequence_number(2),
                Persisted::from(id, Event::B).version(4).sequence_number(3),
                Persisted::from(id, Event::A).version(5).sequence_number(4)
            ]
        );

        // Stream from another offset.
        assert_eq!(
            block_on(stream_to_vec(&store, id, Select::From(4))).unwrap(),
            vec![
                Persisted::from(id, Event::B).version(4).sequence_number(3),
                Persisted::from(id, Event::A).version(5).sequence_number(4)
            ]
        );

        // Stream from an unexistent offset.
        assert!(block_on(stream_to_vec(&store, id, Select::From(10)))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn stream_all() {
        let id_1 = "test-stream-all-1";
        let id_2 = "test-stream-all-2";

        let mut store = InMemoryStore::<&'static str, Event>::default();

        let result = block_on(store.append(id_1, Expected::Any, vec![Event::A]));
        assert!(result.is_ok());

        let result = block_on(store.append(id_2, Expected::Any, vec![Event::B]));
        assert!(result.is_ok());

        let result = block_on(store.append(id_1, Expected::Any, vec![Event::C]));
        assert!(result.is_ok());

        let result = block_on(store.append(id_2, Expected::Any, vec![Event::A]));
        assert!(result.is_ok());

        // Stream from the start.
        let result: anyhow::Result<Vec<Persisted<&str, Event>>> = block_on(async {
            store
                .stream_all(Select::All)
                .await
                .unwrap()
                .try_collect()
                .await
                .map_err(anyhow::Error::from)
        });

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(
            vec![
                Persisted::from(id_1, Event::A)
                    .version(1)
                    .sequence_number(0),
                Persisted::from(id_2, Event::B)
                    .version(1)
                    .sequence_number(1),
                Persisted::from(id_1, Event::C)
                    .version(2)
                    .sequence_number(2),
                Persisted::from(id_2, Event::A)
                    .version(2)
                    .sequence_number(3)
            ],
            result
        );

        // Stream from a specified sequence number.
        let result: anyhow::Result<Vec<Persisted<&str, Event>>> = block_on(async {
            store
                .stream_all(Select::From(2))
                .await
                .unwrap()
                .try_collect()
                .await
                .map_err(anyhow::Error::from)
        });

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(
            vec![
                Persisted::from(id_1, Event::C)
                    .version(2)
                    .sequence_number(2),
                Persisted::from(id_2, Event::A)
                    .version(2)
                    .sequence_number(3)
            ],
            result
        );
    }

    async fn stream_to_vec(
        store: &InMemoryStore<&'static str, Event>,
        id: &'static str,
        select: Select,
    ) -> anyhow::Result<Vec<Persisted<&'static str, Event>>> {
        store
            .stream(id, select)
            .await
            .map_err(anyhow::Error::from)?
            .try_collect()
            .await
            .map_err(anyhow::Error::from)
    }
}
