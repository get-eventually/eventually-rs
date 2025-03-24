use std::time::{SystemTime, UNIX_EPOCH};

use eventually_core::event::store::{self, AppendError, Appender, Streamer};
use eventually_core::event::{Persisted, VersionSelect};
use eventually_core::version::Version;
use eventually_core::{serde, version};
use eventually_postgres::event;
use futures::TryStreamExt;
use rand::Rng;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

mod setup;

#[tokio::test]
async fn append_with_no_version_check_works() {
    let container = Postgres::default()
        .start()
        .await
        .expect("the postgres container should start");

    let (host, port) = futures::try_join!(container.get_host(), container.get_host_port_ipv4(5432))
        .expect("the postgres container should have both a host and a port exposed");

    println!("postgres container is running at {host}:{port}");

    let pool = sqlx::PgPool::connect(&format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        host, port,
    ))
    .await
    .expect("should be able to create a connection with the database");

    let event_store = event::Store::new(pool, serde::Json::<setup::TestDomainEvent>::default())
        .await
        .unwrap();

    let id = rand::rng().random::<i64>();
    let event_stream_id = format!("test-event-stream-{}", id);

    let expected_events = vec![setup::TestDomainEvent::WasCreated {
        id: setup::TestAggregateId(id),
        name: "test something".to_owned(),
        at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }
    .into()];

    let expected_persisted_events: Vec<_> = expected_events
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, event)| Persisted {
            event,
            stream_id: event_stream_id.clone(),
            version: (i + 1) as Version,
        })
        .collect();

    let expected_event_stream_version = expected_events.len() as Version;

    let new_event_stream_version = event_store
        .append(
            event_stream_id.clone(),
            version::Check::Any,
            expected_events,
        )
        .await
        .expect("the event store should append the events");

    assert_eq!(new_event_stream_version, expected_event_stream_version);

    let actual_persisted_events = event_store
        .stream(&event_stream_id, VersionSelect::All)
        .try_collect::<Vec<_>>()
        .await
        .expect("the event store should stream the events back");

    assert_eq!(actual_persisted_events, expected_persisted_events);
}

#[tokio::test]
async fn it_works_with_version_check_for_conflict() {
    let container = Postgres::default()
        .start()
        .await
        .expect("the postgres container should start");

    let (host, port) = futures::try_join!(container.get_host(), container.get_host_port_ipv4(5432))
        .expect("the postgres container should have both a host and a port exposed");

    println!("postgres container is running at {host}:{port}");

    let pool = sqlx::PgPool::connect(&format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        host, port,
    ))
    .await
    .expect("should be able to create a connection with the database");

    let event_store = event::Store::new(pool, serde::Json::<setup::TestDomainEvent>::default())
        .await
        .unwrap();

    let id = rand::rng().random::<i64>();
    let event_stream_id = format!("test-event-stream-{}", id);

    let expected_events = vec![setup::TestDomainEvent::WasCreated {
        id: setup::TestAggregateId(id),
        name: "test something".to_owned(),
        at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }
    .into()];

    let expected_persisted_events: Vec<_> = expected_events
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, event)| Persisted {
            event,
            stream_id: event_stream_id.clone(),
            version: (i + 1) as Version,
        })
        .collect();

    let expected_event_stream_version = expected_events.len() as Version;

    let new_event_stream_version = event_store
        .append(
            event_stream_id.clone(),
            version::Check::MustBe(0),
            expected_events,
        )
        .await
        .expect("the event store should append the events");

    assert_eq!(new_event_stream_version, expected_event_stream_version);

    let actual_persisted_events = event_store
        .stream(&event_stream_id, VersionSelect::All)
        .try_collect::<Vec<_>>()
        .await
        .expect("the event store should stream the events back");

    assert_eq!(actual_persisted_events, expected_persisted_events);

    // Appending twice the with an unexpected Event Stream version should
    // result in a version::ConflictError.
    let error = event_store
        .append(event_stream_id.clone(), version::Check::MustBe(0), vec![])
        .await
        .expect_err("the event store should have returned a conflict error");

    if let AppendError::Conflict(err) = error {
        return assert_eq!(
            err,
            version::ConflictError {
                expected: 0,
                actual: new_event_stream_version,
            }
        );
    }

    panic!("unexpected error received: {}", error);
}

#[tokio::test]
async fn it_handles_concurrent_writes_to_the_same_stream() {
    let container = Postgres::default()
        .start()
        .await
        .expect("the postgres container should start");

    let (host, port) = futures::try_join!(container.get_host(), container.get_host_port_ipv4(5432))
        .expect("the postgres container should have both a host and a port exposed");

    println!("postgres container is running at {host}:{port}");

    let pool = sqlx::PgPool::connect(&format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        host, port,
    ))
    .await
    .expect("should be able to create a connection with the database");

    let event_store = event::Store::new(pool, serde::Json::<setup::TestDomainEvent>::default())
        .await
        .unwrap();

    let id = rand::rng().random::<i64>();
    let event_stream_id = format!("test-event-stream-{}", id);

    let expected_events = vec![setup::TestDomainEvent::WasCreated {
        id: setup::TestAggregateId(id),
        name: "test something".to_owned(),
        at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }
    .into()];

    let result = futures::join!(
        event_store.append(
            event_stream_id.clone(),
            version::Check::MustBe(0),
            expected_events.clone(),
        ),
        event_store.append(
            event_stream_id.clone(),
            version::Check::MustBe(0),
            expected_events,
        )
    );

    match result {
        (Ok(_), Err(store::AppendError::Conflict(_)))
        | (Err(store::AppendError::Conflict(_)), Ok(_)) => {
            // This is the expected scenario :)
        },
        (first, second) => panic!(
            "invalid state detected, first: {:?}, second: {:?}",
            first, second
        ),
    };
}
