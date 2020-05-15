//! Contains supporting entities using an in-memory backend.

use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::Hash;
use std::sync::Arc;

#[cfg(not(feature = "tokio"))]
use std::sync::RwLock;
#[cfg(feature = "tokio")]
use tokio::sync::RwLock;

use futures::future::BoxFuture;
use futures::stream::{empty, iter, BoxStream, StreamExt};

/// An in-memory [`EventStore`] implementation, backed by an [`HashMap`].
///
/// [`EventStore`]: ../../eventually_core/store/trait.EventStore.html
/// [`HashMap`]: something
#[derive(Debug, Clone)]
pub struct EventStore<Id, Event>
where
    Id: Hash + Eq,
{
    backend: Arc<RwLock<HashMap<Id, Vec<Event>>>>,
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
                .and_modify(|vec| vec.extend(events.clone()))
                .or_insert(events);

            Ok(())
        })
    }

    fn stream(
        &self,
        id: Self::SourceId,
        from: Self::Offset,
    ) -> BoxFuture<Result<BoxStream<Result<Self::Event, Self::Error>>, Self::Error>> {
        Box::pin(async move {
            #[cfg(not(feature = "tokio"))]
            let reader = self.backend.read().unwrap();
            #[cfg(feature = "tokio")]
            let reader = self.backend.read().await;

            Ok(reader
                .get(&id)
                .cloned()
                .map(move |events| events_stream_from_offset(from, events))
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

fn events_stream_from_offset<'store, Event>(
    from: usize,
    events: Vec<Event>,
) -> BoxStream<'store, Result<Event, std::convert::Infallible>>
where
    Event: Send + 'store,
{
    let it = events.into_iter().enumerate().filter_map(
        move |(i, event)| {
            if i >= from {
                Some(event)
            } else {
                None
            }
        },
    );

    iter(it).map(Ok).boxed()
}
