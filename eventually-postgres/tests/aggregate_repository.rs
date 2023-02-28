use eventually::{
    aggregate::repository::{GetError, Getter, Saver},
    serde::json::Json,
    version,
};
use eventually_postgres::aggregate;
use futures::TryFutureExt;
use rand::Rng;

mod setup;

#[tokio::test]
async fn it_works() {
    let pool = setup::connect_to_database()
        .await
        .expect("connection to the database should work");

    let aggregate_repository = aggregate::Repository::new(
        pool,
        Json::<setup::TestAggregate>::default(),
        Json::<setup::TestDomainEvent>::default(),
    )
    .await
    .unwrap();

    let aggregate_id = setup::TestAggregateId(rand::thread_rng().gen::<i64>());

    let result = aggregate_repository
        .get(&aggregate_id)
        .await
        .expect_err("should fail");

    match result {
        GetError::NotFound => (),
        _ => panic!(
            "unexpected error received, should be 'not found': {:?}",
            result
        ),
    };

    let mut root = setup::TestAggregateRoot::create(aggregate_id, "John Dee".to_owned())
        .expect("aggregate root should be created");

    // We also delete it just to cause more Domain Events in its Event Stream.
    root.delete().unwrap();

    aggregate_repository
        .save(&mut root)
        .await
        .expect("storing the new aggregate root should be successful");

    let found_root = aggregate_repository
        .get(&aggregate_id)
        .await
        .map(setup::TestAggregateRoot::from)
        .expect("the aggregate root should be found successfully");

    assert_eq!(found_root, root);
}

#[tokio::test]
async fn it_detects_data_races_and_returns_conflict_error() {
    let pool = setup::connect_to_database()
        .await
        .expect("connection to the database should work");

    let aggregate_repository = aggregate::Repository::new(
        pool,
        Json::<setup::TestAggregate>::default(),
        Json::<setup::TestDomainEvent>::default(),
    )
    .await
    .unwrap();

    let aggregate_id = setup::TestAggregateId(rand::thread_rng().gen::<i64>());

    let mut root = setup::TestAggregateRoot::create(aggregate_id, "John Dee".to_owned())
        .expect("aggregate root should be created");

    // We also delete it just to cause more Domain Events in its Event Stream.
    root.delete().unwrap();

    // We clone the Aggregate Root instance so that we have the same
    // uncommitted events list as the original instance.
    let mut cloned_root = root.clone();

    let result = futures::join!(
        aggregate_repository
            .save(&mut root)
            .map_err(Option::<version::ConflictError>::from),
        aggregate_repository
            .save(&mut cloned_root)
            .map_err(Option::<version::ConflictError>::from),
    );

    match result {
        (Ok(()), Err(Some(_))) => (),
        (Err(Some(_)), Ok(())) => (),
        (first, second) => panic!(
            "invalid state detected, first: {:?}, second: {:?}",
            first, second
        ),
    };
}
