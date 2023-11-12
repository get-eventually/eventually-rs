use std::time::{SystemTime, UNIX_EPOCH};

use eventually::event::{Appender, Persisted, StreamVersionExpected, Streamer, VersionSelect};
use eventually::serde::json::Json;
use eventually::version;
use eventually::version::Version;
use eventually_postgres::event;
use futures::{TryFutureExt, TryStreamExt};
use rand::Rng;

mod setup;

#[tokio::test]
async fn append_with_no_version_check_works() {
    let pool = setup::connect_to_database()
        .await
        .expect("connection to the database should work");

    let event_store = event::Store::new(pool, Json::<setup::TestDomainEvent>::default())
        .await
        .unwrap();

    let id = rand::thread_rng().gen::<i64>();
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
            StreamVersionExpected::Any,
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
    let pool = setup::connect_to_database()
        .await
        .expect("connection to the database should work");

    let event_store = event::Store::new(pool, Json::<setup::TestDomainEvent>::default())
        .await
        .unwrap();

    let id = rand::thread_rng().gen::<i64>();
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
            StreamVersionExpected::MustBe(0),
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
    let error: Option<version::ConflictError> = event_store
        .append(
            event_stream_id.clone(),
            StreamVersionExpected::MustBe(0),
            vec![],
        )
        .await
        .expect_err("the event store should have returned a conflict error")
        .into();

    assert_eq!(
        error,
        Some(version::ConflictError {
            expected: 0,
            actual: new_event_stream_version,
        })
    );
}

#[tokio::test]
async fn it_handles_concurrent_writes_to_the_same_stream() {
    let pool = setup::connect_to_database()
        .await
        .expect("connection to the database should work");

    let event_store = event::Store::new(pool, Json::<setup::TestDomainEvent>::default())
        .await
        .unwrap();

    let id = rand::thread_rng().gen::<i64>();
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
            StreamVersionExpected::MustBe(0),
            expected_events.clone(),
        ),
        event_store.append(
            event_stream_id.clone(),
            StreamVersionExpected::MustBe(0),
            expected_events,
        )
    );

    match result {
        (Ok(_), Err(err)) | (Err(err), Ok(_)) => {
            if let event::AppendError::Conflict(_) | event::AppendError::Concurrency(_) = err {
                // This is the expected scenario :)
            } else {
                panic!("unexpected error, {:?}", err);
            }
        },
        (first, second) => panic!(
            "invalid state detected, first: {:?}, second: {:?}",
            first, second
        ),
    };
}
