use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use async_stream::try_stream;

use async_trait::async_trait;

use futures::stream::{empty, StreamExt};

use tokio::sync::broadcast::{channel, error::RecvError, Sender};

use crate::eventstore::{
    EventStore, EventStream, PersistedEvent, PersistedEvents, Select, Version,
};
use crate::Events;

const SUBSCRIBE_CHANNEL_DEFAULT_CAPACITY: usize = 128;

#[derive(Debug, Clone)]
pub struct InMemoryEventStore<Id, Evt> {
    subscribe_capacity: usize,
    events: Arc<RwLock<HashMap<Id, PersistedEvents<Id, Evt>>>>,
    senders: Arc<RwLock<HashMap<Id, Sender<PersistedEvent<Id, Evt>>>>>,
}

impl<Id, Evt> Default for InMemoryEventStore<Id, Evt> {
    #[inline]
    fn default() -> Self {
        Self {
            subscribe_capacity: SUBSCRIBE_CHANNEL_DEFAULT_CAPACITY,
            senders: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "conflict error, last sequence number: `{last_sequence_number}`, caller provided: `{provided}`"
)]
pub struct ConflictError {
    last_sequence_number: u32,
    provided: u32,
}

impl crate::eventstore::ConflictError for ConflictError {
    #[inline]
    fn is_conflict(&self) -> bool {
        true
    }
}

#[async_trait]
impl<Id, Evt> EventStore<Id, Evt> for InMemoryEventStore<Id, Evt>
where
    Id: Eq + Hash + Clone + Send + Sync + Unpin,
    Evt: Clone + Send + Sync + Unpin,
{
    type AppendError = ConflictError;
    type StreamError = std::convert::Infallible;
    type SubscribeError = RecvError;

    async fn append(
        &mut self,
        id: &Id,
        version: Version,
        events: Events<Evt>,
    ) -> Result<u32, Self::AppendError> {
        let last_sequence_number = self
            .events
            .read()
            .unwrap()
            .get(id)
            .and_then(|events| events.last())
            .map(PersistedEvent::sequence_number)
            .unwrap_or_default();

        if let Version::Exact(sequence_number) = version {
            if sequence_number != last_sequence_number {
                return Err(ConflictError {
                    last_sequence_number,
                    provided: sequence_number,
                });
            }
        }

        let new_sequence_number = last_sequence_number + (events.len() as u32);

        let new_events = events
            .into_iter()
            .enumerate()
            .map(|(i, event)| PersistedEvent {
                stream_id: id.clone(),
                sequence_number: (last_sequence_number + 1) + (i as u32),
                event,
            });

        self.events
            .write()
            .unwrap()
            .entry(id.clone())
            .and_modify(|events| events.extend(new_events.clone()))
            .or_insert_with(|| new_events.clone().collect());

        #[allow(unused_must_use)]
        if let Some(tx) = self.senders.read().unwrap().get(id) {
            new_events.for_each(|event| {
                tx.send(event.clone());
            })
        }

        Ok(new_sequence_number)
    }

    fn stream(&self, id: &Id, select: Select) -> EventStream<Id, Evt, Self::StreamError> {
        let events = self.events.read().unwrap();
        let events = events.get(id);

        let no_events = events.map(Vec::is_empty).unwrap_or(true);
        if no_events {
            return Box::pin(empty());
        }

        let events = events.cloned().expect("should have events");

        Box::pin(try_stream! {
            for event in events {
                match select {
                    Select::All => yield event,
                    Select::From(sequence_number) if event.sequence_number() >= sequence_number => {
                        yield event
                    }
                    _ => (),
                };
            }
        })
    }

    fn subscribe(&self, id: &Id) -> EventStream<Id, Evt, Self::SubscribeError> {
        self.senders
            .write()
            .unwrap()
            .entry(id.clone())
            .or_insert_with(|| {
                let (tx, _) = channel(self.subscribe_capacity);
                tx
            })
            .subscribe()
            .into_stream()
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::stream::TryStreamExt;

    use tokio::sync::Notify;

    use crate::Event;

    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    struct MyEvent;

    #[tokio::test]
    async fn stream_works() {
        let mut store = InMemoryEventStore::<&'static str, MyEvent>::default();
        let stream_id = "test-stream-1";

        store
            .append(&stream_id, Version::Any, vec![Event::new(MyEvent)])
            .await
            .expect("no error");

        assert_eq!(
            vec![PersistedEvent {
                stream_id,
                sequence_number: 1,
                event: Event::new(MyEvent),
            }],
            store
                .stream(&stream_id, Select::All)
                .try_collect::<Vec<_>>()
                .await
                .expect("no errors")
        );
    }

    #[tokio::test]
    async fn subscribe_works() {
        let store = InMemoryEventStore::<&'static str, MyEvent>::default();
        let stream_id = "test-subscribe-1";

        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        let rx_beginning = store.subscribe(&stream_id);

        let mut mut_store = store.clone();

        tokio::spawn(async move {
            mut_store
                .append(&stream_id, Version::Any, vec![Event::new(MyEvent)])
                .await
                .expect("no error");

            mut_store
                .append(&stream_id, Version::Any, vec![Event::new(MyEvent)])
                .await
                .expect("no error");

            notify.notify_one();
        });

        notify_clone.notified().await;

        assert_eq!(
            vec![
                PersistedEvent {
                    stream_id,
                    sequence_number: 1,
                    event: Event::new(MyEvent),
                },
                PersistedEvent {
                    stream_id,
                    sequence_number: 2,
                    event: Event::new(MyEvent),
                }
            ],
            drain_stream(rx_beginning, 2).await.expect("no errors")
        );

        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        let rx_after = store.subscribe(&stream_id);

        let mut mut_store = store.clone();

        tokio::spawn(async move {
            mut_store
                .append(&stream_id, Version::Any, vec![Event::new(MyEvent)])
                .await
                .expect("no error");

            notify.notify_one();
        });

        notify_clone.notified().await;

        assert_eq!(
            vec![PersistedEvent {
                stream_id,
                sequence_number: 3,
                event: Event::new(MyEvent),
            }],
            drain_stream(rx_after, 1).await.expect("no errors")
        );
    }

    async fn drain_stream<T, E>(
        stream: impl futures::Stream<Item = Result<T, E>> + Unpin,
        size: usize,
    ) -> Result<Vec<T>, E> {
        let mut stream = stream.enumerate();
        let mut elements = Vec::with_capacity(size);

        while let Some((i, res)) = stream.next().await {
            elements.push(res?);

            if (i + 1) == size {
                break;
            }
        }

        Ok(elements)
    }
}
