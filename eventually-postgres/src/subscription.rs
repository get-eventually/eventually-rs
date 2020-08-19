//! Contains a persisted implementation of the [`Subscription`] trait
//! using Postgres as the backend data source for its state.
//!
//! [`Subscription`]: ../../eventually-core/subscription/trait.Subscription.html

use std::convert::TryFrom;
use std::error::Error as StdError;
use std::fmt::Display;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::{StreamExt, TryStreamExt};

use serde::{Deserialize, Serialize};

use tokio_postgres::Client;

use eventually_core::store::{EventStore as EventStoreTrait, Select};
use eventually_core::subscription::{
    EventSubscriber as EventSubscriberTrait, Subscription, SubscriptionStream,
};

use crate::store::{Error as EventStoreError, EventStore};
use crate::subscriber::{DeserializeError, EventSubscriber};
use crate::Params;

const GET_OR_CREATE_SUBSCRIPTION: &str = "SELECT * FROM get_or_create_subscription($1, $2)";
const CHECKPOINT_SUBSCRIPTION: &str = "SELECT * FROM checkpoint_subscription($1, $2, $3)";

/// Error types returned by a [`Persistent`] Subscription.
///
/// [`Persistent`]: struct.Persistent.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error variant returned when the cause of the error is produced
    /// by the [`EventStore`] during the Subscription stream.
    ///
    /// [`EventStore`]: ../store/struct.EventStore.html
    #[error("error detected while reading one-off event stream from the store: {0}")]
    Store(#[source] EventStoreError),

    // Error variant returned when the cause of the error is produced
    /// by the [`EventSubscriber`] during the Subscription stream.
    ///
    /// [`EventSubscriber`]: ../store/struct.EventSubscriber.html
    #[error("error detected while reading catch-up event stream from the subscription: {0}")]
    Subscriber(#[source] DeserializeError),

    #[error("failed to checkpoint persistent subscription version: {0}")]
    Checkpoint(#[source] tokio_postgres::Error),
}

/// Builder type for multiple [`Persistent`] Subscription instance.
///
/// [`Persistent`]: struct.Persistent.html
pub struct PersistentBuilder<SourceId, Event> {
    client: Arc<Client>,
    store: EventStore<SourceId, Event>,
    subscriber: EventSubscriber<SourceId, Event>,
}

impl<SourceId, Event> PersistentBuilder<SourceId, Event>
where
    SourceId: Clone,
    Event: Clone,
{
    /// Creates a new [`PersistentBuilder`] instance.
    ///
    /// [`PersistentBuilder`]: struct.PersistentBuilder.html
    pub fn new(
        client: Arc<Client>,
        store: EventStore<SourceId, Event>,
        subscriber: EventSubscriber<SourceId, Event>,
    ) -> Self {
        Self {
            client,
            store,
            subscriber,
        }
    }

    /// Creates a new [`Persisted`] Subscription with the specified name, if it
    /// doesn't exists already.
    ///
    /// If a Subscription exists with the specified name, loads the state
    /// from the database and returns the [`Persisted`] instance
    /// pointing to the that state.
    ///
    /// [`Persisted`]: struct.Persisted.html
    pub async fn get_or_create(
        &self,
        name: String,
    ) -> Result<Persistent<SourceId, Event>, tokio_postgres::Error> {
        let params: Params = &[&name, &self.store.type_name];

        let row = self
            .client
            .query_one(GET_OR_CREATE_SUBSCRIPTION, params)
            .await?;

        let last_sequence_number: i64 = row.try_get("last_sequence_number")?;

        Ok(Persistent {
            name,
            last_sequence_number: AtomicI64::from(last_sequence_number),
            client: self.client.clone(),
            store: self.store.clone(),
            subscriber: self.subscriber.clone(),
        })
    }
}

/// [`Subscription`] type with persistent state over a Postgres
/// data source.
///
/// Use [`PersistentBuilder`] to create new instances of this type.
///
/// [`PersistentBuilder`]: struct.PersistentBuilder.html
/// [`Subscription`]: ../../eventually-core/subscription/trait.Subscription.html
pub struct Persistent<SourceId, Event> {
    name: String,
    last_sequence_number: AtomicI64,
    client: Arc<Client>,
    store: EventStore<SourceId, Event>,
    subscriber: EventSubscriber<SourceId, Event>,
}

impl<SourceId, Event> Subscription for Persistent<SourceId, Event>
where
    SourceId: TryFrom<String> + Display + Eq + Clone + Send + Sync,
    Event: Serialize + Clone + Send + Sync,
    for<'de> Event: Deserialize<'de>,
    <SourceId as TryFrom<String>>::Error: StdError + Send + Sync + 'static,
{
    type SourceId = SourceId;
    type Event = Event;
    type Error = Error;

    fn resume(&self) -> BoxFuture<Result<SubscriptionStream<Self>, Self::Error>> {
        Box::pin(async move {
            // Get the next item from the last processed one.
            //
            // In the initial case, the last_sequence_number
            // would be -1, which will load everything from the start.
            let checkpoint = (self.last_sequence_number.load(Ordering::Relaxed) as u32) + 1;

            tracing::debug!(
                subscription.checkpoint = checkpoint,
                subscription.name = %self.name,
                subscription.aggregate_type = %self.store.type_name,
                "Resuming persistent subscription"
            );

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
            let subscription = self
                .subscriber
                .subscribe_all()
                .await
                .map_err(Error::Subscriber)?;

            let one_off_stream = self
                .store
                .stream_all(Select::From(checkpoint))
                .await
                .map_err(Error::Store)?;

            let stream = one_off_stream
                .map_err(Error::Store)
                .chain(subscription.map_err(Error::Subscriber))
                .try_filter_map(move |event| async move {
                    let event_sequence_number = event.sequence_number() as i64;
                    let expected_sequence_number =
                        self.last_sequence_number.load(Ordering::Relaxed);

                    if event_sequence_number <= expected_sequence_number {
                        tracing::debug!(
                            event.sequence_number = event_sequence_number,
                            expected = expected_sequence_number,
                            "Duplicated event detected"
                        );

                        return Ok(None); // Duplicated event detected, let's skip it.
                    }

                    Ok(Some(event))
                })
                .boxed();

            Ok(stream)
        })
    }

    fn checkpoint(&self, version: u32) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            let params: Params = &[&self.name, &self.store.type_name, &(version as i64)];

            tracing::debug!(
                checkpoint = version,
                subscription.name = %self.name,
                subscription.aggregate_type = %self.store.type_name,
                "Checkpointing persistent subscription"
            );

            self.client
                .execute(CHECKPOINT_SUBSCRIPTION, params)
                .await
                .map_err(Error::Checkpoint)?;

            self.last_sequence_number
                .store(version as i64, Ordering::Relaxed);

            Ok(())
        })
    }
}
