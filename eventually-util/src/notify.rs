use std::fmt::Debug;

use eventually_core::store::{EventStore, EventStream};

use futures::future::BoxFuture;

use tokio::sync::broadcast::{channel, Receiver, Sender};

pub trait EventStoreExt: EventStore + Sized {
    #[inline]
    fn subscribable(self) -> Notifier<Self> {
        Notifier::new(self)
    }
}

impl<T> EventStoreExt for T where T: EventStore + Sized {}

pub type AppendEvent<S> = (<S as EventStore>::SourceId, Vec<<S as EventStore>::Event>);

#[derive(Debug, Clone)]
pub struct Notifier<Store>
where
    Store: EventStore,
{
    event_store: Store,
    append_tx: Sender<AppendEvent<Store>>,
    remove_tx: Sender<Store::SourceId>,
}

impl<Store> Notifier<Store>
where
    Store: EventStore,
{
    fn new(store: Store) -> Self {
        let (append_tx, _) = channel::<AppendEvent<Store>>(10);
        let (remove_tx, _) = channel::<Store::SourceId>(10);

        Self {
            event_store: store,
            append_tx,
            remove_tx,
        }
    }

    #[inline]
    pub fn subscribe_append(&self) -> Receiver<AppendEvent<Store>> {
        self.append_tx.subscribe()
    }

    #[inline]
    pub fn subscribe_remove(&self) -> Receiver<Store::SourceId> {
        self.remove_tx.subscribe()
    }
}

impl<Store> EventStore for Notifier<Store>
where
    Store: EventStore + Send + Sync,
    Store::SourceId: Debug + Clone + Send + Sync,
    Store::Event: Debug + Clone + Send + Sync,
{
    type SourceId = Store::SourceId;
    type Offset = Store::Offset;
    type Event = Store::Event;
    type Error = Store::Error;

    fn append(
        &mut self,
        id: Self::SourceId,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            self.event_store.append(id.clone(), events.clone()).await?;
            self.append_tx.send((id, events)).unwrap();

            Ok(())
        })
    }

    fn stream(
        &self,
        id: Self::SourceId,
        from: Self::Offset,
    ) -> BoxFuture<Result<EventStream<Self>, Self::Error>> {
        self.event_store.stream(id, from)
    }

    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            self.event_store.remove(id.clone()).await?;
            self.remove_tx.send(id).unwrap();

            Ok(())
        })
    }
}
