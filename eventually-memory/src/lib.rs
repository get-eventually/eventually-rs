use std::{
    collections::HashMap,
    convert::Infallible,
    hash::Hash,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use futures::stream::{empty, iter, BoxStream, StreamExt};

use eventually_core::store::Store;

#[derive(Clone)]
pub struct MemoryStore<SourceId, Event> {
    store: Arc<RwLock<HashMap<SourceId, Vec<Event>>>>,
}

impl<SourceId, Event> Default for MemoryStore<SourceId, Event>
where
    SourceId: Hash + Eq,
{
    fn default() -> Self {
        MemoryStore::new()
    }
}

impl<SourceId, Event> MemoryStore<SourceId, Event>
where
    SourceId: Hash + Eq,
{
    pub fn new() -> Self {
        MemoryStore {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<SourceId, Event> Store for MemoryStore<SourceId, Event>
where
    SourceId: Hash + Eq + Send + Sync,
    Event: Clone + Send + Sync + 'static,
{
    type SourceId = SourceId;
    type Offset = usize;
    type Event = Event;
    type Error = Infallible;

    fn stream(
        &self,
        source_id: Self::SourceId,
        from: Self::Offset,
    ) -> BoxStream<'_, Result<Self::Event, Infallible>> {
        self.store
            .read()
            .unwrap()
            .get(&source_id)
            .cloned()
            .map(move |events| events_stream_from_offset(from, events))
            .unwrap_or_else(|| empty().boxed())
    }

    async fn append(
        &mut self,
        source_id: Self::SourceId,
        events: Vec<Self::Event>,
    ) -> Result<(), Self::Error> {
        self.store
            .write()
            .unwrap()
            .entry(source_id)
            .and_modify(|vec| {
                vec.extend(events.clone());
            })
            .or_insert(events);

        Ok(())
    }
}

fn events_stream_from_offset<Event>(
    from: usize,
    events: Vec<Event>,
) -> BoxStream<'static, Result<Event, Infallible>>
where
    Event: Send + 'static,
{
    iter(events.into_iter().enumerate().filter_map(
        move |(idx, event)| {
            if idx >= from {
                Some(event)
            } else {
                None
            }
        },
    ))
    .map(Result::Ok)
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::stream::StreamExt;

    #[derive(Debug, PartialEq, Eq, Clone)]
    enum Event {
        A,
        B,
        C,
    }

    #[test]
    fn it_works() {
        let mut store = MemoryStore::<&'static str, Event>::default();

        tokio_test::block_on(store.append("stream1", vec![Event::A, Event::B, Event::C])).unwrap();

        assert_eq!(
            tokio_test::block_on(
                store
                    .stream("stream1", 0)
                    .map(|result| result.unwrap())
                    .collect::<Vec<Event>>()
            ),
            vec![Event::A, Event::B, Event::C]
        );

        tokio_test::block_on(store.append("stream1", vec![Event::B, Event::C, Event::A])).unwrap();

        assert_eq!(
            tokio_test::block_on(
                store
                    .stream("stream1", 0)
                    .map(|result| result.unwrap())
                    .collect::<Vec<Event>>()
            ),
            vec![Event::A, Event::B, Event::C, Event::B, Event::C, Event::A]
        );

        assert_eq!(
            tokio_test::block_on(
                store
                    .stream("stream1", 3)
                    .map(|result| result.unwrap())
                    .collect::<Vec<Event>>()
            ),
            vec![Event::B, Event::C, Event::A]
        );

        assert_eq!(
            tokio_test::block_on(
                store
                    .stream("stream1", 7)
                    .map(|result| result.unwrap())
                    .collect::<Vec<Event>>()
            ),
            vec![]
        );
    }
}
