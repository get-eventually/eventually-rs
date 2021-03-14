//! Contains a persisted implementation of the [`Subscription`] trait
//! using Postgres as the backend data source for its state.
//!
//! [`Subscription`]: ../../eventually-core/subscription/trait.Subscription.html

use std::convert::{TryFrom, TryInto};
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::sync::atomic::{AtomicI64, Ordering};

use futures::future::BoxFuture;
use futures::stream::{StreamExt, TryStreamExt};

use serde::{Deserialize, Serialize};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::Socket;

use eventually_core::store::{EventStore as EventStoreTrait, Select};
use eventually_core::subscription::{
    EventSubscriber as EventSubscriberTrait, Subscription, SubscriptionStream,
};

use crate::store::{Error as EventStoreError, EventStore, PoolResult};
use crate::subscriber::{EventSubscriber, SubscriberError};
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

    /// Error variant returned when the cause of the error is produced
    /// by the [`EventSubscriber`] during the Subscription stream.
    ///
    /// [`EventSubscriber`]: ../store/struct.EventSubscriber.html
    #[error("error detected while reading catch-up event stream from the subscription: {0}")]
    Subscriber(#[source] SubscriberError),

    /// Error variant returned when an issue has occurred during the checkpoint
    /// of a processed event.
    #[error("failed to checkpoint persistent subscription version: {0}")]
    Checkpoint(#[source] bb8::RunError<tokio_postgres::Error>),
}

/// Builder type for multiple [`Persistent`] Subscription instance.
///
/// [`Persistent`]: struct.Persistent.html
pub struct PersistentBuilder<SourceId, Event, Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    pool: Pool<PostgresConnectionManager<Tls>>,
    store: EventStore<SourceId, Event, Tls>,
    subscriber: EventSubscriber<SourceId, Event>,
}

impl<SourceId, Event, Tls> PersistentBuilder<SourceId, Event, Tls>
where
    SourceId: Clone,
    Event: Clone,
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    /// Creates a new [`PersistentBuilder`] instance.
    ///
    /// [`PersistentBuilder`]: struct.PersistentBuilder.html
    pub fn new(
        pool: Pool<PostgresConnectionManager<Tls>>,
        store: EventStore<SourceId, Event, Tls>,
        subscriber: EventSubscriber<SourceId, Event>,
    ) -> Self {
        Self {
            pool,
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
    ) -> PoolResult<Persistent<SourceId, Event, Tls>> {
        let params: Params = &[&name, &self.store.type_name];

        let client = self.pool.get().await?;
        let row = client.query_one(GET_OR_CREATE_SUBSCRIPTION, params).await?;

        let last_sequence_number: i64 = row.try_get("last_sequence_number")?;

        Ok(Persistent {
            name,
            last_sequence_number: AtomicI64::from(last_sequence_number),
            pool: self.pool.clone(),
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
pub struct Persistent<SourceId, Event, Tls>
where
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    name: String,
    last_sequence_number: AtomicI64,
    pool: Pool<PostgresConnectionManager<Tls>>,
    store: EventStore<SourceId, Event, Tls>,
    subscriber: EventSubscriber<SourceId, Event>,
}

impl<SourceId, Event, Tls> Subscription for Persistent<SourceId, Event, Tls>
where
    SourceId: TryFrom<String> + Display + Eq + Clone + Send + Sync + 'static,
    Event: Serialize + Clone + Send + Sync + Debug + 'static,
    for<'de> Event: Deserialize<'de>,
    <SourceId as TryFrom<String>>::Error: StdError + Send + Sync + 'static,
    Tls: MakeTlsConnect<Socket> + Clone + Send + Sync + 'static,
    <Tls as MakeTlsConnect<Socket>>::Stream: Send + Sync,
    <Tls as MakeTlsConnect<Socket>>::TlsConnect: Send,
    <<Tls as MakeTlsConnect<Socket>>::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    type SourceId = SourceId;
    type Event = Event;
    type Error = Error;

    fn resume(&self) -> BoxFuture<Result<SubscriptionStream<Self>, Self::Error>> {
        let fut = async move {
            let last_sequence_number = self.last_sequence_number.load(Ordering::Relaxed);

            // Get the next item from the last processed one.
            //
            // In the initial case, the last_sequence_number
            // would be -1, which will load everything from the start.
            let checkpoint: u32 = (last_sequence_number + 1).try_into().expect(
                "in case of overflow, it means there is a bug in the optimistic versioning code; \\
                please open an issue with steps to reproduce the bug",
            );

            #[cfg(feature = "with-tracing")]
            tracing::trace!(
                subscription.sequence_number = last_sequence_number,
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
                        #[cfg(feature = "with-tracing")]
                        tracing::trace!(
                            event.sequence_number = event_sequence_number,
                            subscription.sequence_number = expected_sequence_number,
                            "Duplicated event detected; skipping"
                        );

                        return Ok(None);
                    }

                    Ok(Some(event))
                })
                .boxed();

            Ok(stream)
        };

        Box::pin(fut)
    }

    fn checkpoint(&self, version: u32) -> BoxFuture<Result<(), Self::Error>> {
        Box::pin(async move {
            let params: Params = &[&self.name, &self.store.type_name, &(version as i64)];

            #[cfg(feature = "with-tracing")]
            tracing::trace!(
                checkpoint = version,
                subscription.name = %self.name,
                subscription.aggregate_type = %self.store.type_name,
                "Checkpointing persistent subscription"
            );

            let client = self.pool.get().await.map_err(Error::Checkpoint)?;
            client
                .execute(CHECKPOINT_SUBSCRIPTION, params)
                .await
                .map_err(bb8::RunError::User)
                .map_err(Error::Checkpoint)?;

            self.last_sequence_number
                .store(version as i64, Ordering::Relaxed);

            Ok(())
        })
    }
}
