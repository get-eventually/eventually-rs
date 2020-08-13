use std::fmt::Debug;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use eventually_core::projection::Projection;
use eventually_core::store::{EventStore, Select};
use eventually_core::subscription::EventSubscriber;

use futures::stream::{Stream, StreamExt, TryStreamExt};

use tokio::sync::watch::{channel, Receiver, Sender};

pub struct ProjectorBuilder<Store, Subscriber> {
    store: Arc<Store>,
    subscriber: Arc<Subscriber>,
}

impl<Store, Subscriber> ProjectorBuilder<Store, Subscriber> {
    pub fn new(store: Arc<Store>, subscriber: Arc<Subscriber>) -> Self {
        Self { store, subscriber }
    }

    pub fn build<P>(&self) -> Projector<P, Store, Subscriber>
    where
        P: Projection + Debug + Clone,
        Store: EventStore<SourceId = P::SourceId, Event = P::Event>,
        Subscriber: EventSubscriber<SourceId = P::SourceId, Event = P::Event>,
        <Store as EventStore>::Error: std::error::Error + Send + Sync + 'static,
        <Subscriber as EventSubscriber>::Error: std::error::Error + Send + Sync + 'static,
    {
        Projector::new(self.store.clone(), self.subscriber.clone())
    }
}

pub struct Projector<P, Store, Subscriber>
where
    P: Projection,
{
    tx: Sender<P>,
    rx: Receiver<P>, // Keep the receiver to be able to clone it in watch().
    store: Arc<Store>,
    subscriber: Arc<Subscriber>,
    state: P,
    last_sequence_number: AtomicU32,
    projection: std::marker::PhantomData<P>,
}

impl<P, Store, Subscriber> Projector<P, Store, Subscriber>
where
    P: Projection + Debug + Clone,
    Store: EventStore<SourceId = P::SourceId, Event = P::Event>,
    Subscriber: EventSubscriber<SourceId = P::SourceId, Event = P::Event>,
    // NOTE: this bound is needed to clone the current state for next projection.
    // NOTE: these bounds are needed for anyhow::Error conversion.
    <Store as EventStore>::Error: std::error::Error + Send + Sync + 'static,
    <Subscriber as EventSubscriber>::Error: std::error::Error + Send + Sync + 'static,
{
    fn new(store: Arc<Store>, subscriber: Arc<Subscriber>) -> Self {
        let state: P = Default::default();
        let (tx, rx) = channel(state.clone());

        Self {
            tx,
            rx,
            store,
            subscriber,
            state,
            last_sequence_number: Default::default(),
            projection: std::marker::PhantomData,
        }
    }

    pub fn watch(&self) -> impl Stream<Item = P> {
        self.rx.clone()
    }

    pub async fn run(&mut self, select: Select) -> anyhow::Result<()> {
        // Create the Subscription first, so that once the future has been resolved
        // we'll start receiving events right away.
        //
        // This is to avoid losing events when waiting for the one-off stream
        // to resolve its future.
        //
        // The impact is that we _might_ get duplicated events from the one-off stream
        // and the subscription stream. Luckily, we can discard those by
        // keeping an internal state of the last processed sequence number,
        // and discard all those events that are found.
        let subscription = self.subscriber.subscribe_all().await?;
        let one_off_stream = self.store.stream_all(select).await?;

        let mut stream = one_off_stream
            .map_err(anyhow::Error::from)
            .chain(subscription.map_err(anyhow::Error::from));

        while let Some(event) = stream.next().await {
            let event = event?;
            let expected_sequence_number = self.last_sequence_number.load(Ordering::SeqCst);
            let event_sequence_number = event.sequence_number();

            // If some bounds are requested when running the projector,
            // make sure the subscription is also upholding the Select operation
            // by skipping events with a sequence number we're not interested in.
            if let Select::From(v) = select {
                if event_sequence_number < v {
                    continue;
                }
            }

            if event_sequence_number < expected_sequence_number {
                continue; // Duplicated event detected, let's skip it.
            }

            self.state = P::project(self.state.clone(), event);

            self.last_sequence_number.compare_and_swap(
                expected_sequence_number,
                event_sequence_number,
                Ordering::SeqCst,
            );

            // Notify watchers of the latest projection state.
            self.tx.broadcast(self.state.clone()).expect(
                "since this struct holds the original receiver, failures should not happen",
            );
        }

        Ok(())
    }
}
