//! Redis backend implementation for [`eventually` crate](https://crates.io/crates/eventually).
//!
//! ## Event Store
//!
//! `eventually-redis` supports the [`eventually::EventStore`] trait through
//! the [`EventStore`] type.
//!
//! ## Event Subscriber
//!
//! `eventually-redis` supports the [`eventually::EventSubscriber`] trait
//! through the [`EventSubscriber`] type.
//!
//! [`eventually::EventStore`]: ../eventually/trait.EventStore.html
//! [`EventStore`]: struct.EventStore.html
//! [`EventSubscriber`]: struct.EventSubscriber.html

mod store;
mod stream;
mod subscriber;
mod subscription;

pub use store::*;
pub use subscriber::*;
pub use subscription::*;

use redis::RedisResult;

/// Default size of a paginated request to Redis `XRANGE .. COUNT n`
/// for the [`EventStore::stream`] and [`EventStore::stream_all`] operations.
///
/// Page size can be overridden through the [`EventStoreBuilder::stream_page_size`]
/// option.
///
/// [`EventStore::stream`]: struct.EventStore.html#tymethod.stream
/// [`EventStore::stream_all`]: struct.EventStore.html#tymethod.stream_all
/// [`EventStoreBuilder::stream_page_size`]: struct.EventStoreBuilder.html#method.stream_page_size
pub const STREAM_PAGE_DEFAULT: usize = 128;

/// Builder type for [`EventStore`] and [`EventSubscriber`] types.
///
/// The same builder instance can be used to build multiple instances of [`EventStore`]
/// and [`EventSubscriber`].
///
/// [`EventStore`]: struct.EventStore.html
/// [`EventSubscriber`]: struct.EventSubscriber.html
#[derive(Clone)]
pub struct Builder {
    client: redis::Client,
    stream_page_size: Option<usize>,
}

impl Builder {
    /// Creates a new builder instance using the specified Redis client.
    pub fn new(client: redis::Client) -> Self {
        Self {
            client,
            stream_page_size: None,
        }
    }

    /// Changes the page size used by the [`Stream`] returned in [`EventStore::stream`]
    /// and [`EventStore::stream_all`].
    ///
    /// [`Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
    /// [`EventStore::stream`]: struct.EventStore.html#tymethod.stream
    /// [`EventStore::stream_all`]: struct.EventStore.html#tymethod.stream_all
    pub fn stream_page_size(mut self, size: usize) -> Self {
        self.stream_page_size = Some(size);
        self
    }

    /// Assignes the specified source name to a copy of the current builder
    /// instance, in order to create [`EventStore`] and [`EventSubscriber`]
    /// for the same source type identifier (e.g. the Aggregate type identifier).
    ///
    /// [`EventStore`]: struct.EventStore.html
    /// [`EventSubscriber`]: struct.EventSubscriber.html
    pub fn source_name(&self, name: &'static str) -> BuilderWithSourceName {
        BuilderWithSourceName {
            client: self.client.clone(),
            stream_page_size: self.stream_page_size,
            source_name: name,
        }
    }
}

/// Second-step builder type for [`EventStore`] and [`EventSubscriber`] types.
///
/// This particular builder type has a source name (i.e. an aggregate type name)
/// specified from [`Builder::source_name`].
///
/// [`Builder::source_name`]: struct.Builder.html#method.source_name
/// [`EventStore`]: struct.EventStore.html
/// [`EventSubscriber`]: struct.EventSubscriber.html
#[derive(Clone)]
pub struct BuilderWithSourceName {
    client: redis::Client,
    source_name: &'static str,
    stream_page_size: Option<usize>,
}

impl BuilderWithSourceName {
    /// Changes the page size used by the [`Stream`] returned in [`EventStore::stream`]
    /// and [`EventStore::stream_all`].
    ///
    /// [`Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
    /// [`EventStore::stream`]: struct.EventStore.html#tymethod.stream
    /// [`EventStore::stream_all`]: struct.EventStore.html#tymethod.stream_all
    pub fn stream_page_size(mut self, size: usize) -> Self {
        self.stream_page_size = Some(size);
        self
    }

    /// Builds a new [`EventStore`] instance.
    ///
    /// This method returns an `std::future::Future` completing after a
    /// connection with Redis is successfully established.
    ///
    /// [`EventStore`]: struct.EventStore.html
    pub async fn build_store<Id, Event>(&self) -> RedisResult<EventStore<Id, Event>> {
        Ok(EventStore {
            stream_name: self.source_name,
            conn: self.client.get_multiplexed_async_connection().await?,
            stream_page_size: self.stream_page_size.unwrap_or(STREAM_PAGE_DEFAULT),
            id: std::marker::PhantomData,
            event: std::marker::PhantomData,
        })
    }

    /// Builds a new [`EventSubscriber`] instance.
    ///
    /// [`EventSubscriber`]: struct.EventSubscriber.html
    pub fn build_subscriber<Id, Event>(&self) -> EventSubscriber<Id, Event> {
        EventSubscriber {
            stream_name: self.source_name,
            client: self.client.clone(),
            id: std::marker::PhantomData,
            event: std::marker::PhantomData,
        }
    }

    /// Builds a new named [`PersistentSubscription`] instance.
    ///
    /// [`PersistentSubscription`]: struct.PersistentSubscription.html
    pub async fn build_persistent_subscription<Id, Event>(
        &self,
        subscription_name: &'static str,
    ) -> RedisResult<PersistentSubscription<Id, Event>> {
        let mut subscription = PersistentSubscription {
            stream: self.source_name,
            group_name: subscription_name,
            conn: self.client.get_multiplexed_async_connection().await?,
            stream_page_size: self.stream_page_size.unwrap_or(STREAM_PAGE_DEFAULT),
            id: std::marker::PhantomData,
            event: std::marker::PhantomData,
        };

        subscription.create_consumer_group().await?;

        Ok(subscription)
    }
}
