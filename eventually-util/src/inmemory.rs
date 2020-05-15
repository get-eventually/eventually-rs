//! Contains supporting entities using an in-memory backend.

use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;
use std::sync::Arc;

use eventually_core::aggregate::{Aggregate, AggregateId};
use eventually_core::store::EventStream;

use futures::future::BoxFuture;
use futures::stream::{empty, iter, StreamExt};

#[cfg(not(feature = "tokio"))]
use std::sync::RwLock;
#[cfg(feature = "tokio")]
use tokio::sync::RwLock;

#[derive(Debug)]
struct EventsHolder<Event> {
    global_offset: usize,
    events: Vec<(usize, Vec<Event>)>,
}

impl<Event> From<Vec<Event>> for EventsHolder<Event> {
    fn from(events: Vec<Event>) -> Self {
        let mut blocks = Vec::new();
        blocks.push((1, events));

        Self {
            global_offset: 1,
            events: blocks,
        }
    }
}

impl<Event> EventsHolder<Event>
where
    Event: Clone,
{
    fn append(&mut self, events: Vec<Event>) {
        let offset = self.global_offset + 1;

        self.events.push((offset, events));
        self.global_offset = offset;
    }

    fn stream(&self, from: usize) -> impl Iterator<Item = Event> {
        self.events
            .clone() // Should be moved inside the block flat-mapping if possible
            .into_iter()
            .filter(move |(offset, _)| offset >= &from)
            .flat_map(|(_, events)| events)
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
    pub fn for_aggregate<T>(_: &T) -> EventStore<AggregateId<T>, T::Event>
    where
        T: Aggregate,
        AggregateId<T>: Hash + Eq,
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
    backend: Arc<RwLock<HashMap<Id, EventsHolder<Event>>>>,
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
    type Offset = usize;
    type Event = Event;
    type Error = Infallible;

    fn append(
        &mut self,
        id: Self::SourceId,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            #[cfg(not(feature = "tokio"))]
            let mut writer = self.backend.write().unwrap();
            #[cfg(feature = "tokio")]
            let mut writer = self.backend.write().await;

            writer
                .entry(id)
                .and_modify(|holder| holder.append(events.clone()))
                .or_insert_with(|| EventsHolder::from(events));

            Ok(())
        })
    }

    fn stream(
        &self,
        id: Self::SourceId,
        from: Self::Offset,
    ) -> BoxFuture<Result<EventStream<Self>, Self::Error>> {
        Box::pin(async move {
            #[cfg(not(feature = "tokio"))]
            let reader = self.backend.read().unwrap();
            #[cfg(feature = "tokio")]
            let reader = self.backend.read().await;

            Ok(reader
                .get(&id)
                .map(move |holder| iter(holder.stream(from)).map(Ok).boxed())
                .unwrap_or_else(|| empty().boxed()))
        })
    }

    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            #[cfg(not(feature = "tokio"))]
            let mut writer = self.backend.write().unwrap();
            #[cfg(feature = "tokio")]
            let mut writer = self.backend.write().await;

            writer.remove(&id);

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::EventStore as InMemoryStore;
    use eventually_core::store::EventStore;

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
        let combined = vec![Event::A, Event::B, Event::C, Event::B, Event::A];

        // Offset 1
        assert!(block_on(store.append("test-stream", events_1.clone())).is_ok());
        // Offset 2
        assert!(block_on(store.append("test-stream", events_2.clone())).is_ok());

        assert_eq!(
            combined,
            block_on(stream_to_vec_from(&store, "test-stream", 1))
        );

        assert_eq!(
            events_2,
            block_on(stream_to_vec_from(&store, "test-stream", 2))
        );
    }

    fn append_to(store: &mut InMemoryStore<&'static str, Event>, id: &'static str) {
        let events = vec![Event::A, Event::B, Event::C];

        assert!(block_on(store.append(id, events.clone())).is_ok());
        assert_eq!(events, block_on(stream_to_vec_from(&store, id, 0)));
    }

    async fn stream_to_vec_from(
        store: &InMemoryStore<&'static str, Event>,
        id: &'static str,
        from: usize,
    ) -> Vec<Event> {
        store
            .stream(id, from)
            .await
            .expect("should be infallible")
            .map(|event| event.expect("should be infallible"))
            .collect::<Vec<Event>>()
            .await
    }
}
