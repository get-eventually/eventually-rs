use std::time::{SystemTime, UNIX_EPOCH};

use eventually::{
    event::{Persisted, Store, StreamVersionExpected, VersionSelect},
    serde::json::Json,
    version::Version,
};
use eventually_postgres::event;
use futures::stream::TryStreamExt;
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

    let event_stream_id = format!("test-event-stream-{}", rand::thread_rng().gen::<u64>());

    let expected_events = vec![setup::TestDomainEvent::WasCreated {
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
