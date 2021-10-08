//! Contains supporting entities using an in-memory backend.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::aggregate::Aggregate;
use crate::store::persistent::EventBuilderWithVersion;
use crate::store::{AppendError, EventStream, Expected, Persisted, Select};
use crate::subscription::EventSubscriber;
use crate::versioning::Versioned;

use futures::future::BoxFuture;
use futures::stream::{empty, iter, StreamExt, TryStreamExt};

use parking_lot::RwLock;

use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

#[cfg(feature = "with-tracing")]
use tracing_futures::Instrument;

const SUBSCRIBE_CHANNEL_DEFAULT_CAP: usize = 128;

/// Error returned by the
/// [`EventStore::append`](crate::store::EventStore) when a conflict
/// has been detected.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("conflicting versions, expected {expected}, got instead {actual}")]
pub struct ConflictError {
    /// The last version value found the Store.
    pub expected: u32,
    /// The actual version passed by the caller to the Store.
    pub actual: u32,
}

impl AppendError for ConflictError {
    fn is_conflict_error(&self) -> bool {
        true
    }
}

/// Error returned by the [`EventSubscriber`] when reading elements
/// from the [`EventStream`] produced by
/// [`subscribe_all`](EventSubscriber::subscribe_all).
#[derive(Debug, thiserror::Error)]
#[error("failed to read event from subscription watch channel: receiver lagged messages {0}")]
pub struct LaggedError(u64);

/// Builder for [`EventStore`] instances.
pub struct EventStoreBuilder;

impl EventStoreBuilder {
    /// Builds a new [`EventStore`] instance compatible with the provided
    /// [`Aggregate`].
    #[inline]
    pub fn for_aggregate<T>(_: &T) -> EventStore<T::Id, T::Event>
    where
        T: Aggregate,
        T::Id: Hash + Eq + Clone,
        T::Event: Clone,
    {
        EventStore::default()
    }
}

type Backend<Id, Event> = HashMap<Id, Vec<Persisted<Id, Event>>>;

/// An in-memory [`EventStore`] implementation, backed by an [`HashMap`].
#[derive(Debug, Clone)]
pub struct EventStore<Id, Event>
where
    Id: Hash + Eq,
{
    global_offset: Arc<AtomicU32>,
    tx: Sender<Persisted<Id, Event>>,
    backend: Arc<RwLock<Backend<Id, Event>>>,
}

impl<Id, Event> EventStore<Id, Event>
where
    Id: Hash + Eq + Clone,
    Event: Clone,
{
    /// Creates a new [`EventStore`] with a specified in-memory broadcast channel
    /// size, which will used by the
    /// [`subscribe_all`](EventSubscriber::subscribe_all) method to notify
    /// of newly [`EventStore::append`](crate::store::EventStore)
    /// events.
    pub fn new(subscribe_capacity: usize) -> Self {
        // Use this broadcast channel to send append events to
        // subscriptions from .subscribe_all()
        let (tx, _rx) = channel(subscribe_capacity);

        Self {
            tx,
            global_offset: Arc::new(AtomicU32::new(0)),
            backend: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<Id, Event> Default for EventStore<Id, Event>
where
    Id: Hash + Eq + Clone,
    Event: Clone,
{
    #[inline]
    fn default() -> Self {
        Self::new(SUBSCRIBE_CHANNEL_DEFAULT_CAP)
    }
}

impl<Id, Event> EventSubscriber for EventStore<Id, Event>
where
    Id: Hash + Eq + Sync + Send + Clone + 'static,
    Event: Sync + Send + Clone + 'static,
{
    type SourceId = Id;
    type Event = Event;
    type Error = LaggedError;

    fn subscribe_all(&self) -> crate::subscription::EventStream<Self> {
        // Create a new Receiver from the store Sender.
        //
        // This receiver implements the TryStream trait, which works perfectly
        // with the definition of the EventStream.
        let rx = self.tx.subscribe();

        BroadcastStream::new(rx)
            .map_err(|v| match v {
                BroadcastStreamRecvError::Lagged(n) => LaggedError(n),
            })
            .boxed()
    }
}

impl<Id, Event> crate::store::EventStore for EventStore<Id, Event>
where
    Id: Hash + Eq + Sync + Send + Debug + Clone,
    Event: Sync + Send + Debug + Clone,
{
    type SourceId = Id;
    type Event = Event;
    type Error = ConflictError;

    fn append(
        &mut self,
        id: Self::SourceId,
        version: Expected,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<u32, Self::Error>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::append",
            id = ?id,
            version = ?version,
            events = ?events
        );

        let fut = async move {
            let expected = self
                .backend
                .read()
                .get(&id)
                .and_then(|events| events.last())
                .map_or(0, Versioned::version);

            if let Expected::Exact(actual) = version {
                if expected != actual {
                    return Err(ConflictError { expected, actual });
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

            // Copy of the events for broadcasting.
            let broadcast_copy = persisted_events.clone();

            let last_version = persisted_events.last().map_or(expected, Persisted::version);

            self.backend
                .write()
                .entry(id)
                .and_modify(|events| events.append(&mut persisted_events))
                .or_insert_with(|| persisted_events);

            // From tokio documentation, the send() operation can only
            // fail if there are no active receivers to listen to these events.
            //
            // From our perspective, this is not an issue, since appends
            // might be done without any active subscription.
            #[allow(unused_must_use)]
            {
                // Broadcast events into the store's Sender channel.
                for event in broadcast_copy {
                    self.tx.send(event);
                }
            }

            Ok(last_version)
        };

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
    }

    fn stream(
        &self,
        id: Self::SourceId,
        select: Select,
    ) -> BoxFuture<Result<EventStream<Self>, Self::Error>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::stream",
            id = ?id,
            select = ?select
        );

        let fut = async move {
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
        };

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
    }

    fn stream_all(&self, select: Select) -> BoxFuture<Result<EventStream<Self>, Self::Error>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::stream_all",
            select = ?select
        );

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
        events.sort_by_key(Persisted::sequence_number);

        let fut = futures::future::ok(iter(events).map(Ok).boxed());

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
    }

    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>> {
        #[cfg(feature = "with-tracing")]
        let span = tracing::info_span!(
            "EventStore::remove",
            id = ?id
        );

        let fut = async move {
            self.backend.write().remove(&id);

            Ok(())
        };

        #[cfg(feature = "with-tracing")]
        let fut = fut.instrument(span);

        Box::pin(fut)
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
    use super::{ConflictError, EventStore as InMemoryStore};

    use std::cell::RefCell;
    use std::sync::Arc;

    use crate::store::{EventStore, Expected, Persisted, Select};
    use crate::subscription::EventSubscriber;

    use futures::{StreamExt, TryStreamExt};

    use tokio::sync::Barrier;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum Event {
        A,
        B,
        C,
    }

    #[tokio::test]
    async fn subscribe_returns_all_the_latest_events() {
        let id_1 = "test-subscribe-1";
        let id_2 = "test-subscribe-2";

        // Create the store and the clone to move into the async closure.
        let store = InMemoryStore::<&'static str, Event>::default();
        let store_1 = store.clone();

        // Use a barrier to synchronize the start of the first 2 events
        // and the first subscription start.
        let barrier = Arc::new(Barrier::new(2));
        let barrier_1 = barrier.clone();

        // First subscription.
        let join_handle_1 = tokio::spawn(async move {
            let mut events = store_1.subscribe_all().enumerate();
            barrier_1.wait().await;

            while let Some((i, res)) = events.next().await {
                assert!(res.is_ok());
                let event = res.unwrap();

                match i {
                    0 => assert_eq!(
                        Persisted::from(id_1, Event::A)
                            .version(1)
                            .sequence_number(0),
                        event
                    ),
                    1 => assert_eq!(
                        Persisted::from(id_1, Event::B)
                            .version(2)
                            .sequence_number(1),
                        event
                    ),
                    2 => assert_eq!(
                        Persisted::from(id_1, Event::C)
                            .version(3)
                            .sequence_number(2),
                        event
                    ),
                    3 => {
                        assert_eq!(
                            Persisted::from(id_2, Event::A)
                                .version(1)
                                .sequence_number(3),
                            event
                        );
                        // Break out of the stream looping after the last expected
                        // event has been received.
                        break;
                    }
                    _ => panic!("should not reach this point"),
                };
            }
        });

        // Use internal mutability to escape the borrow checker rules for the test,
        // because append() requires &mut self, but subscribe() requires &self first.
        let store = RefCell::new(store);
        barrier.wait().await;

        assert!(store
            .borrow_mut()
            .append(id_1, Expected::Exact(0), vec![Event::A])
            .await
            .is_ok());

        assert!(store
            .borrow_mut()
            .append(id_1, Expected::Exact(1), vec![Event::B])
            .await
            .is_ok());

        let store = store.into_inner();
        let store_2 = store.clone();

        // Same thing as above, but to wait the second batch of the events
        // appended to the store.
        let barrier = Arc::new(Barrier::new(2));
        let barrier_2 = barrier.clone();

        // Second subscriber, it will only see events of the second batch,
        // which is when it started listening to events.
        let join_handle_2 = tokio::spawn(async move {
            let mut events = store_2.subscribe_all().enumerate();
            barrier_2.wait().await;

            while let Some((i, res)) = events.next().await {
                assert!(res.is_ok());
                let event = res.unwrap();

                match i {
                    0 => assert_eq!(
                        Persisted::from(id_1, Event::C)
                            .version(3)
                            .sequence_number(2),
                        event
                    ),
                    1 => {
                        assert_eq!(
                            Persisted::from(id_2, Event::A)
                                .version(1)
                                .sequence_number(3),
                            event
                        );
                        break;
                    }
                    _ => panic!("should not reach this point"),
                };
            }
        });

        let store = RefCell::new(store);
        barrier.wait().await;

        assert!(store
            .borrow_mut()
            .append(id_1, Expected::Exact(2), vec![Event::C])
            .await
            .is_ok());

        assert!(store
            .borrow_mut()
            .append(id_2, Expected::Exact(0), vec![Event::A])
            .await
            .is_ok());

        // Wait for both subscribers to be done.
        tokio::try_join!(join_handle_1, join_handle_2).unwrap();
    }

    #[tokio::test]
    async fn append_with_any_versions_works() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        let id = "test-append";

        let events = vec![Event::A, Event::B, Event::C];

        assert!(store
            .append(id, Expected::Any, events.clone())
            .await
            .is_ok());

        assert!(store
            .append(id, Expected::Any, events.clone())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn append_with_expected_versions_works() {
        let mut store = InMemoryStore::<&'static str, Event>::default();
        let id = "test-append";

        let events = vec![Event::A, Event::B, Event::C];

        let result = store.append(id, Expected::Exact(0), events.clone()).await;
        assert!(result.is_ok());

        let last_version = result.unwrap();

        assert!(store
            .append(id, Expected::Exact(last_version), events.clone())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn append_with_wrong_expected_versions_fails() {
        let id = "test-append";
        let mut store = InMemoryStore::<&'static str, Event>::default();

        let events = vec![Event::A, Event::B, Event::C];

        let result = store.append(id, Expected::Exact(0), events.clone()).await;
        assert!(result.is_ok());

        let last_version = result.unwrap();
        let poisoned_last_version = last_version + 1; // Poison the last version on purpose

        let result = store
            .append(id, Expected::Exact(poisoned_last_version), events.clone())
            .await;

        assert_eq!(
            Err(ConflictError {
                expected: last_version,
                actual: poisoned_last_version
            }),
            result
        );
    }

    #[tokio::test]
    async fn remove() {
        let id = "test-remove";
        let mut store = InMemoryStore::<&'static str, Event>::default();

        // Removing an empty stream works.
        assert!(store.remove(id).await.is_ok());
        assert!(stream_to_vec(&store, id, Select::All)
            .await
            .unwrap()
            .is_empty());

        // Add some events and lets remove them after
        let events = vec![Event::A, Event::B, Event::C];
        assert!(store
            .append(id, Expected::Exact(0), events.clone())
            .await
            .is_ok());

        assert!(store.remove(id).await.is_ok());
        assert!(stream_to_vec(&store, id, Select::All)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn stream() {
        let id = "test-stream";
        let mut store = InMemoryStore::<&'static str, Event>::default();

        let events_1 = vec![Event::A, Event::B, Event::C];
        let events_2 = vec![Event::B, Event::A];

        let result = store.append(id, Expected::Exact(0), events_1).await;
        assert!(result.is_ok());

        let last_version = result.unwrap();
        assert!(store
            .append(id, Expected::Exact(last_version), events_2)
            .await
            .is_ok());

        // Stream from the start.
        assert_eq!(
            stream_to_vec(&store, id, Select::All).await.unwrap(),
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
            stream_to_vec(&store, id, Select::From(4)).await.unwrap(),
            vec![
                Persisted::from(id, Event::B).version(4).sequence_number(3),
                Persisted::from(id, Event::A).version(5).sequence_number(4)
            ]
        );

        // Stream from an unexistent offset.
        assert!(stream_to_vec(&store, id, Select::From(10))
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn stream_all() {
        let id_1 = "test-stream-all-1";
        let id_2 = "test-stream-all-2";

        let mut store = InMemoryStore::<&'static str, Event>::default();

        assert!(store
            .append(id_1, Expected::Any, vec![Event::A])
            .await
            .is_ok());

        assert!(store
            .append(id_2, Expected::Any, vec![Event::B])
            .await
            .is_ok());

        assert!(store
            .append(id_1, Expected::Any, vec![Event::C])
            .await
            .is_ok());

        assert!(store
            .append(id_2, Expected::Any, vec![Event::A])
            .await
            .is_ok());

        // Stream from the start.
        let result: anyhow::Result<Vec<Persisted<&str, Event>>> = store
            .stream_all(Select::All)
            .await
            .unwrap()
            .try_collect()
            .await
            .map_err(anyhow::Error::from);

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
        let result: anyhow::Result<Vec<Persisted<&str, Event>>> = store
            .stream_all(Select::From(2))
            .await
            .unwrap()
            .try_collect()
            .await
            .map_err(anyhow::Error::from);

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
