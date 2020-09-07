use std::sync::Arc;

use eventually_core::store::{EventStore, Expected, Persisted};
use eventually_core::subscription::{EventSubscriber as EventSubscriberTrait, Subscription};
use eventually_postgres::{EventStoreBuilder, EventSubscriber, PersistentBuilder};

use futures::stream::{StreamExt, TryStreamExt};

use serde::{Deserialize, Serialize};

use testcontainers::core::Docker;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum Event {
    A,
    B,
    C,
}

#[tokio::test]
async fn subscribe_all_works() {
    let docker = testcontainers::clients::Cli::default();
    let postgres_image = testcontainers::images::postgres::Postgres::default();
    let node = docker.run(postgres_image);

    let dsn = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let (mut client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("could not connect to the docker container");

    tokio::spawn(async move {
        connection
            .await
            .expect("connection with the database exited with error")
    });

    let source_name = "subscriber_test";
    let source_id = "subscriber_test";

    let event_store_builder = EventStoreBuilder::migrate_database(&mut client)
        .await
        .expect("failed to run database migrations")
        .builder(Arc::new(client));

    let mut event_store = event_store_builder
        .build::<String, Event>(source_name)
        .await
        .expect("failed to create event store");

    let event_subscriber = EventSubscriber::<String, Event>::new(&dsn, source_name)
        .await
        .expect("failed to create event subscription");

    let subscription = event_subscriber
        .subscribe_all()
        .await
        .expect("failed to create subscription from event subscriber");

    event_store
        .append(
            source_id.to_owned(),
            Expected::Exact(0),
            vec![Event::A, Event::B, Event::C],
        )
        .await
        .expect("failed while appending events");

    let events: Vec<Persisted<String, Event>> = subscription
        .take(3)
        .try_collect()
        .await
        .expect("failed to collect events from subscription");

    assert_eq!(
        vec![
            Persisted::from(source_id.to_owned(), Event::A)
                .version(1)
                .sequence_number(0),
            Persisted::from(source_id.to_owned(), Event::B)
                .version(2)
                .sequence_number(1),
            Persisted::from(source_id.to_owned(), Event::C)
                .version(3)
                .sequence_number(2)
        ],
        events
    );
}

#[tokio::test]
async fn persistent_subscription_works() {
    let docker = testcontainers::clients::Cli::default();
    let postgres_image = testcontainers::images::postgres::Postgres::default();
    let node = docker.run(postgres_image);

    let dsn = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let (mut client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("could not connect to the docker container");

    tokio::spawn(async move {
        connection
            .await
            .expect("connection with the database exited with error")
    });

    let source_name = "persistent_subscription_test";
    let source_id = "persistent_subscription_test";

    let event_store_builder = EventStoreBuilder::migrate_database(&mut client)
        .await
        .expect("failed to run database migrations")
        .builder(Arc::new(client));

    let mut event_store = event_store_builder
        .build::<String, Event>(source_name)
        .await
        .expect("failed to create event store");

    let event_subscriber = EventSubscriber::<String, Event>::new(&dsn, source_name)
        .await
        .expect("failed to create event subscription");

    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("could not connect to the docker container");

    tokio::spawn(async move {
        connection
            .await
            .expect("connection with the database exited with error")
    });

    let subscription =
        PersistentBuilder::new(Arc::new(client), event_store.clone(), event_subscriber)
            .get_or_create(source_name.to_owned())
            .await
            .expect("failed to create persistent subscription");

    event_store
        .append(
            source_id.to_owned(),
            Expected::Exact(0),
            vec![Event::A, Event::B, Event::C],
        )
        .await
        .expect("failed while appending events");

    let events: Vec<Persisted<String, Event>> = subscription
        .resume()
        .await
        .expect("failed to resume subscription")
        .take(3)
        .try_collect()
        .await
        .expect("failed to collect events from subscription");

    assert_eq!(
        vec![
            Persisted::from(source_id.to_owned(), Event::A)
                .version(1)
                .sequence_number(0),
            Persisted::from(source_id.to_owned(), Event::B)
                .version(2)
                .sequence_number(1),
            Persisted::from(source_id.to_owned(), Event::C)
                .version(3)
                .sequence_number(2)
        ],
        events
    );
}
