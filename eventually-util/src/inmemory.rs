//! Contains supporting entities using an in-memory backend.

use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;
use std::sync::Arc;

use eventually_core::aggregate::Aggregate;
use eventually_core::store::{EventStream, PersistedEvent, Select};

use futures::future::BoxFuture;
use futures::stream::{empty, iter, StreamExt};

use parking_lot::RwLock;

fn into_persisted_events<T>(version: u32, events: Vec<T>) -> Vec<PersistedEvent<T>> {
    events
        .into_iter()
        .enumerate()
        .map(|(i, event)| {
            PersistedEvent::from(event)
                .with_version(version)
                .with_sequence_number(i as u32)
        })
        .collect()
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
    backend: Arc<RwLock<HashMap<Id, Vec<PersistedEvent<Event>>>>>,
}

impl<Id, Event> Default for EventStore<Id, Event>
where
    Id: Hash + Eq,
{
    #[inline]
    fn default() -> Self {
        Self {
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
    type Error = Infallible;

    fn append(
        &mut self,
        id: Self::SourceId,
        version: u32,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            let mut persisted_events = into_persisted_events(version, events);

            self.backend
                .write()
                .entry(id)
                .and_modify(|events| events.append(&mut persisted_events))
                .or_insert_with(|| persisted_events);

            Ok(())
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

#[cfg(test)]
mod tests {
    use super::EventStore as InMemoryStore;
    use eventually_core::store::{EventStore, PersistedEvent, Select};

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
        append_to(&mut store, "test-append", 1);
    }

    #[test]
    fn remove() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        append_to(&mut store, "test-remove", 1);

        assert!(block_on(store.remove("test-remove")).is_ok());
        assert!(block_on(stream_to_vec_from(&store, "test-remove", 0)).is_empty());
    }

    #[test]
    fn stream() {
        let mut store = InMemoryStore::<&'static str, Event>::default();

        let events_1 = vec![Event::A, Event::B, Event::C];
        let events_2 = vec![Event::B, Event::A];

        assert!(block_on(store.append("test-stream", 1, events_1.clone())).is_ok());
        assert!(block_on(store.append("test-stream", 2, events_2.clone())).is_ok());

        assert_eq!(
            block_on(stream_to_vec_from(&store, "test-stream", 1)),
            vec![
                PersistedEvent::from(Event::A)
                    .with_version(1)
                    .with_sequence_number(0),
                PersistedEvent::from(Event::B)
                    .with_version(1)
                    .with_sequence_number(1),
                PersistedEvent::from(Event::C)
                    .with_version(1)
                    .with_sequence_number(2),
                PersistedEvent::from(Event::B)
                    .with_version(2)
                    .with_sequence_number(0),
                PersistedEvent::from(Event::A)
                    .with_version(2)
                    .with_sequence_number(1)
            ]
        );

        assert_eq!(
            block_on(stream_to_vec_from(&store, "test-stream", 2)),
            vec![
                PersistedEvent::from(Event::B)
                    .with_version(2)
                    .with_sequence_number(0),
                PersistedEvent::from(Event::A)
                    .with_version(2)
                    .with_sequence_number(1)
            ]
        );
    }

    fn append_to(store: &mut InMemoryStore<&'static str, Event>, id: &'static str, version: u32) {
        let events = vec![Event::A, Event::B, Event::C];

        assert!(block_on(store.append(id, version, events.clone())).is_ok());
        assert_eq!(
            block_on(stream_to_vec_from(&store, id, 0)),
            vec![
                PersistedEvent::from(Event::A)
                    .with_version(1)
                    .with_sequence_number(0),
                PersistedEvent::from(Event::B)
                    .with_version(1)
                    .with_sequence_number(1),
                PersistedEvent::from(Event::C)
                    .with_version(1)
                    .with_sequence_number(2)
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
