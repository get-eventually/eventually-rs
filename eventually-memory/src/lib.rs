use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures::{
    future::{ok, Ready},
    stream::{iter, pending, BoxStream, StreamExt},
};

use eventually::store::{ReadStore, WriteStore};

pub struct MemoryStore<SourceId, Event> {
    store: Arc<RwLock<HashMap<SourceId, Vec<Event>>>>,
}

impl<SourceId, Event> Default for MemoryStore<SourceId, Event>
where
    SourceId: std::hash::Hash + Eq,
{
    fn default() -> Self {
        MemoryStore {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<SourceId, Event> ReadStore for MemoryStore<SourceId, Event>
where
    SourceId: std::hash::Hash + Eq,
    Event: Clone + Send + 'static,
{
    type SourceId = SourceId;
    type Offset = usize;
    type Event = Event;
    type Stream = BoxStream<'static, Self::Event>;

    fn stream(&self, source_id: Self::SourceId, from: Self::Offset) -> Self::Stream {
        let store = self.store.read().unwrap();

        store
            .get(&source_id)
            .cloned()
            .map(move |events| {
                iter(
                    events
                        .into_iter()
                        .enumerate()
                        .filter_map(
                            move |(idx, event)| if idx >= from { Some(event) } else { None },
                        ),
                )
                .boxed()
            })
            .unwrap_or_else(|| pending().boxed())
    }
}

impl<SourceId, Event> WriteStore for MemoryStore<SourceId, Event>
where
    SourceId: std::hash::Hash + Eq,
    Event: Clone,
{
    type SourceId = SourceId;
    type Offset = usize;
    type Event = Event;
    type Error = std::convert::Infallible;
    type Result = Ready<Result<(), Self::Error>>;

    fn append(
        &mut self,
        source_id: Self::SourceId,
        _from: Self::Offset,
        events: Vec<Self::Event>,
    ) -> Self::Result {
        let mut store = self.store.write().unwrap();

        store
            .entry(source_id)
            .and_modify(|vec| {
                vec.extend(events.clone());
            })
            .or_insert(events);

        ok(())
    }
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

        tokio_test::block_on(store.append(
            "stream1",
            0 as usize,
            vec![Event::A, Event::B, Event::C],
        ))
        .unwrap();

        assert_eq!(
            tokio_test::block_on(store.stream("stream1", 0).collect::<Vec<Event>>()),
            vec![Event::A, Event::B, Event::C]
        );

        tokio_test::block_on(store.append(
            "stream1",
            0 as usize,
            vec![Event::B, Event::C, Event::A],
        ))
        .unwrap();

        assert_eq!(
            tokio_test::block_on(store.stream("stream1", 0).collect::<Vec<Event>>()),
            vec![Event::A, Event::B, Event::C, Event::B, Event::C, Event::A]
        );

        assert_eq!(
            tokio_test::block_on(store.stream("stream1", 3).collect::<Vec<Event>>()),
            vec![Event::B, Event::C, Event::A]
        );

        assert_eq!(
            tokio_test::block_on(store.stream("stream1", 7).collect::<Vec<Event>>()),
            vec![]
        );
    }
}
