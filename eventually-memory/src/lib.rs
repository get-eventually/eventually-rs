//! Utility crate for [`eventually`] persistance using in-memory backends.
//!
//! Take a look at [`Store`] for further information.
//!
//! [`eventually`]: ../../eventually
//! [`Store`]: struct.Store.html

use std::{
    collections::HashMap,
    convert::Infallible,
    hash::Hash,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use futures::stream::{empty, iter, BoxStream, StreamExt};

use eventually_core::store::Store as EventStore;

/// A general in-memory, event store implementation compatible with
/// the [`eventually-core::store::Store`] trait.
///
/// The event store is backed by an [`HashMap`] and a linear [`Vec`] of Events,
/// and concurrent accesses are protected by an [`RwLock`].
///
/// # Note
///
/// The event store is safe to use across threads.
///
/// Every instance use [`std::sync::Arc`] internally, so it's also safe
/// to `clone()`.
///
/// # Usage
///
/// ```rust
/// use tokio_test::block_on;
///
/// use futures::stream::StreamExt;
///
/// use eventually_core::store::Store;
/// use eventually_memory::Store as MemoryStore;
///
/// let mut store = MemoryStore::<&'static str, u32>::new();
///
/// // Append all the events: in this case, age changes from 1 to 2
/// block_on(store.append("my-age", vec![1])).unwrap();
/// block_on(store.append("my-age", vec![2])).unwrap();
///
/// // Retrieve all the events from the very start (offset 0):
/// // the list of events contains all the ages we appended before.
/// let result = block_on(
///     store.stream("my-age", 0)
///         .map(|result| result.unwrap())
///         .collect::<Vec<u32>>()
/// );
///
/// assert_eq!(result, vec![1, 2]);
/// ```
///
/// [`eventually-core::store::Store`]: ../../eventually-core/store/trait.Store.html
/// [`HashMap`]: https://docs.rs/std/collections/struct.HashMap.html
/// [`Vec`]: https://docs.rs/std/collections/struct.Vec.html
#[derive(Clone)]
pub struct Store<SourceId, Event> {
    store: Arc<RwLock<HashMap<SourceId, Vec<Event>>>>,
}

impl<SourceId, Event> Default for Store<SourceId, Event>
where
    SourceId: Hash + Eq,
{
    fn default() -> Self {
        Store::new()
    }
}

impl<SourceId, Event> Store<SourceId, Event>
where
    SourceId: Hash + Eq,
{
    /// Creates a new, empty instance of a Store.
    pub fn new() -> Self {
        Store {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<SourceId, Event> EventStore for Store<SourceId, Event>
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
            .and_modify(|vec| vec.extend(events.clone()))
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
        let mut store = Store::<&'static str, Event>::default();

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
