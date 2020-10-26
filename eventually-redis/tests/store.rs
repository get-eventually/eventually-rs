use eventually_core::store::{EventStore, Expected, Select};
use eventually_core::subscription::EventSubscriber;
use eventually_core::versioning::Versioned;
use eventually_redis::{EventStore as RedisEventStore, EventStoreBuilder};

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

    let builder = EventStoreBuilder::new(client).stream_page_size(128);
    let builder_clone = builder.clone();

    tokio::spawn(async move {
        builder_clone
            .build_subscriber::<String, Event>("test-stream")
            .subscribe_all()
            .await
            .unwrap()
            .inspect_err(|err| eprintln!("Failed: {}", err))
            .try_for_each(|event| async {
                Ok(println!(
                    "Seq.no: {}, Version: {}, Event: {:?}",
                    event.sequence_number(),
                    event.version(),
                    event.take(),
                ))
            })
            .await
            .unwrap();
    });

    let mut store: RedisEventStore<String, Event> = builder
        .build_store("test-stream")
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
