use eventually_core::store::{EventStore, Expected, Persisted, Select};
use eventually_postgres::EventStoreBuilder;

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
async fn different_types_can_share_id() {
    let docker = testcontainers::clients::Cli::default();
    let postgres_image = testcontainers::images::postgres::Postgres::default();
    let node = docker.run(postgres_image);

    let dsn = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let pg_manager =
        bb8_postgres::PostgresConnectionManager::new_from_stringlike(&dsn, tokio_postgres::NoTls)
            .expect("Could not parse the dsn string");
    let pool = bb8::Pool::builder()
        .build(pg_manager)
        .await
        .expect("Could not build the pool");

    let event_store_builder = EventStoreBuilder::migrate_database(pool.clone())
        .await
        .expect("failed to run database migrations")
        .builder(pool);

    let event_name = "first_name";
    let ivent_name = "second_name";

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    enum Ivent {
        A(usize),
    }

    let mut event_store = event_store_builder
        .build::<String, Event, _>(event_name)
        .await
        .expect("Failed to create event store");
    let mut ivent_store = event_store_builder
        .build::<String, Ivent, _>(ivent_name)
        .await
        .expect("Failed to create ivent store");
    let shared_id = "first!";
    event_store
        .append(
            shared_id.to_owned(),
            Expected::Exact(0),
            vec![Event::A, Event::B],
        )
        .await
        .expect("Failed appending events");
    ivent_store
        .append(shared_id.to_owned(), Expected::Exact(0), vec![Ivent::A(1)])
        .await
        .expect("Failed appending ivents");
    let ivents: Vec<Persisted<String, Ivent>> = ivent_store
        .stream_all(Select::All)
        .await
        .expect("failed to create first stream")
        .try_collect()
        .await
        .expect("failed to collect ivents from subscription");
    let events: Vec<Persisted<String, Event>> = event_store
        .stream_all(Select::All)
        .await
        .expect("failed to create second stream")
        .try_collect()
        .await
        .expect("failed to collect events from subscription");
    assert_eq!(
        vec![Persisted::from(shared_id.to_owned(), Ivent::A(1))
            .version(1)
            .sequence_number(2),],
        ivents
    );
    assert_eq!(
        vec![
            Persisted::from(shared_id.to_owned(), Event::A)
                .version(1)
                .sequence_number(0),
            Persisted::from(shared_id.to_owned(), Event::B)
                .version(2)
                .sequence_number(1),
        ],
        events
    );
}

#[tokio::test]
async fn stream_all_works() {
    let docker = testcontainers::clients::Cli::default();
    let postgres_image = testcontainers::images::postgres::Postgres::default();
    let node = docker.run(postgres_image);

    let dsn = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let pg_manager =
        bb8_postgres::PostgresConnectionManager::new_from_stringlike(&dsn, tokio_postgres::NoTls)
            .expect("Could not parse the dsn string");
    let pool = bb8::Pool::builder()
        .build(pg_manager)
        .await
        .expect("Could not build the pool");

    let event_store_builder = EventStoreBuilder::migrate_database(pool.clone())
        .await
        .expect("failed to run database migrations")
        .builder(pool);
    let source_name = "stream_all_test";
    let source_id_1 = "stream_all_test_1";
    let source_id_2 = "stream_all_test_2";

    let mut event_store = event_store_builder
        .build::<String, Event, _>(source_name)
        .await
        .expect("failed to create event store");

    event_store
        .append(
            source_id_1.to_owned(),
            Expected::Exact(0),
            vec![Event::A, Event::B, Event::C],
        )
        .await
        .expect("failed while appending events");

    event_store
        .append(
            source_id_2.to_owned(),
            Expected::Exact(0),
            vec![Event::C, Event::A, Event::B],
        )
        .await
        .expect("failed while appending events");

    // Select::All returns all the events.
    let events: Vec<Persisted<String, Event>> = event_store
        .stream_all(Select::All)
        .await
        .expect("failed to create first stream")
        .try_collect()
        .await
        .expect("failed to collect events from subscription");

    assert_eq!(
        vec![
            Persisted::from(source_id_1.to_owned(), Event::A)
                .version(1)
                .sequence_number(0),
            Persisted::from(source_id_1.to_owned(), Event::B)
                .version(2)
                .sequence_number(1),
            Persisted::from(source_id_1.to_owned(), Event::C)
                .version(3)
                .sequence_number(2),
            Persisted::from(source_id_2.to_owned(), Event::C)
                .version(1)
                .sequence_number(3),
            Persisted::from(source_id_2.to_owned(), Event::A)
                .version(2)
                .sequence_number(4),
            Persisted::from(source_id_2.to_owned(), Event::B)
                .version(3)
                .sequence_number(5)
        ],
        events
    );

    // Select::From returns a slice of the events by their sequence number,
    // in this case it will return only events coming from the second source.
    let events: Vec<Persisted<String, Event>> = event_store
        .stream_all(Select::From(3))
        .await
        .expect("failed to create second stream")
        .try_collect()
        .await
        .expect("failed to collect events from subscription");

    assert_eq!(
        vec![
            Persisted::from(source_id_2.to_owned(), Event::C)
                .version(1)
                .sequence_number(3),
            Persisted::from(source_id_2.to_owned(), Event::A)
                .version(2)
                .sequence_number(4),
            Persisted::from(source_id_2.to_owned(), Event::B)
                .version(3)
                .sequence_number(5)
        ],
        events
    );
}

#[tokio::test]
async fn stream_works() {
    let source_name = "stream_test";
    let source_id = "stream_test";
    let docker = testcontainers::clients::Cli::default();
    let postgres_image = testcontainers::images::postgres::Postgres::default();
    let node = docker.run(postgres_image);

    let dsn = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        node.get_host_port(5432).unwrap()
    );

    let pg_manager =
        bb8_postgres::PostgresConnectionManager::new_from_stringlike(&dsn, tokio_postgres::NoTls)
            .expect("Could not parse the dsn string");
    let pool = bb8::Pool::builder()
        .build(pg_manager)
        .await
        .expect("Could not build the pool");

    let event_store_builder = EventStoreBuilder::migrate_database(pool.clone())
        .await
        .expect("failed to run database migrations")
        .builder(pool);

    let mut event_store = event_store_builder
        .build::<String, Event, _>(source_name)
        .await
        .expect("failed to create event store");

    event_store
        .append(
            source_id.to_owned(),
            Expected::Exact(0),
            vec![Event::A, Event::B, Event::C],
        )
        .await
        .expect("failed while appending events");

    // Select::All returns all the events.
    let events: Vec<Persisted<String, Event>> = event_store
        .stream(source_id.to_owned(), Select::All)
        .await
        .expect("failed to create first stream")
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

    // Select::From returns a slice of the events by their version.
    let events: Vec<Persisted<String, Event>> = event_store
        .stream(source_id.to_owned(), Select::From(3))
        .await
        .expect("failed to create second stream")
        .try_collect()
        .await
        .expect("failed to collect events from subscription");

    assert_eq!(
        vec![Persisted::from(source_id.to_owned(), Event::C)
            .version(3)
            .sequence_number(2)],
        events
    );
}
