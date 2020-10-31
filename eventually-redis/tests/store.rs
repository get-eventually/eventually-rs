use std::sync::Arc;

use eventually_core::projection::Projection;
use eventually_core::store::{EventStore, Expected, Persisted, Select};
use eventually_core::subscription::EventSubscriber;
use eventually_redis::{Builder, EventStore as RedisEventStore};
use eventually_util::inmemory::Projector;
use eventually_util::sync::RwLock;

use futures::future::BoxFuture;
use futures::stream::TryStreamExt;

use serde::{Deserialize, Serialize};

use testcontainers::core::Docker;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum Event {
    A,
    B,
    C,
}

#[tokio::test]
async fn it_works() {
    let docker = testcontainers::clients::Cli::default();
    let redis_image = testcontainers::images::redis::Redis::default();
    let node = docker.run(redis_image);

    let dsn = format!("redis://127.0.0.1:{}/", node.get_host_port(6379).unwrap());
    let client = redis::Client::open(dsn).expect("failed to connect to Redis");

    let builder = Builder::new(client)
        .stream_page_size(128)
        .source_name("test-stream");

    let builder_clone = builder.clone();

    tokio::spawn(async move {
        builder_clone
            .build_subscriber::<String, Event>()
            .subscribe_all()
            .await
            .unwrap()
            .try_for_each(|_event| async { Ok(()) })
            .await
            .unwrap();
    });

    let mut store: RedisEventStore<String, Event> = builder
        .build_store()
        .await
        .expect("failed to create redis connection");

    for chunk in 0..1000 {
        store
            .append(
                "test-source-1".to_owned(),
                Expected::Any,
                vec![Event::A, Event::B, Event::C],
            )
            .await
            .unwrap();

        store
            .append(
                "test-source-2".to_owned(),
                Expected::Exact(chunk * 2),
                vec![Event::B, Event::A],
            )
            .await
            .unwrap();
    }

    let now = std::time::SystemTime::now();

    assert_eq!(
        5000usize,
        store
            .stream_all(Select::All)
            .await
            .unwrap()
            .try_fold(0usize, |acc, _x| async move { Ok(acc + 1) })
            .await
            .unwrap()
    );

    println!("Stream $all took {:?}", now.elapsed().unwrap());

    assert_eq!(
        3000usize,
        store
            .stream("test-source-1".to_owned(), Select::All)
            .await
            .unwrap()
            .try_fold(0usize, |acc, _x| async move { Ok(acc + 1) })
            .await
            .unwrap()
    );

    assert_eq!(
        2000usize,
        store
            .stream("test-source-2".to_owned(), Select::All)
            .await
            .unwrap()
            .try_fold(0usize, |acc, _x| async move { Ok(acc + 1) })
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn it_creates_persistent_subscription_successfully() {
    let docker = testcontainers::clients::Cli::default();
    let redis_image = testcontainers::images::redis::Redis::default();
    let node = docker.run(redis_image);

    let dsn = format!("redis://127.0.0.1:{}/", node.get_host_port(6379).unwrap());
    let client = redis::Client::open(dsn).expect("failed to connect to Redis");

    let events_count: usize = 3;

    let builder = Builder::new(client)
        .stream_page_size(128)
        .source_name("test-stream");

    let builder_clone = builder.clone();

    // First copy should create the stream.
    let subscription = builder
        .build_persistent_subscription::<String, Event>("subscription")
        .await
        .unwrap();

    // Second copy should not fail from issuing `XGROUP CREATE` twice.
    builder
        .build_persistent_subscription::<String, Event>("subscription")
        .await
        .unwrap();

    // Create a counter projection of the number of events received.
    #[derive(Debug, Default)]
    struct Counter(usize);
    impl Projection for Counter {
        type SourceId = String;
        type Event = Event;
        type Error = std::convert::Infallible;

        fn project<'a>(
            &'a mut self,
            _event: Persisted<Self::SourceId, Self::Event>,
        ) -> BoxFuture<'a, Result<(), Self::Error>> {
            Box::pin(async move { Ok(self.0 += 1) })
        }
    }

    let counter = Arc::new(RwLock::new(Counter::default()));
    let mut projector = Projector::new(counter.clone(), subscription);

    // Run projection asynchronously.
    tokio::spawn(async move {
        projector.run().await.unwrap();
    });

    // Create entries in the store synchronously, then check for the result
    // of the counter.
    let mut store = builder_clone
        .build_store::<String, Event>()
        .await
        .expect("create event store");

    for i in 0..events_count {
        store
            .append(
                format!("test-subscription-{}", i),
                Expected::Any,
                vec![Event::A, Event::B, Event::C],
            )
            .await
            .unwrap();
    }

    // Get size of committed events from the Event Store.
    let size = store
        .stream_all(Select::All)
        .await
        .unwrap()
        .try_fold(0usize, |acc, _x| async move { Ok(acc + 1) })
        .await
        .unwrap();

    // Wait to make sure enough time has been passed for the subscription
    // stream to pick up other messages from the consumer group.
    tokio::time::delay_for(tokio::time::Duration::from_millis(100)).await;

    assert_eq!(size, counter.read().await.0);
}
