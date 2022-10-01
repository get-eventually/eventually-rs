use eventually::{
    aggregate::{Repository, RepositoryGetError},
    serde::json::Json,
};
use eventually_postgres::aggregate;
use rand::Rng;

mod setup;

#[tokio::test]
async fn add_new_unexisting_aggregate_root() {
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
        RepositoryGetError::AggregateRootNotFound => (),
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
        .store(&mut root)
        .await
        .expect("storing the new aggregate root should be successful");

    let found_root = aggregate_repository
        .get(&aggregate_id)
        .await
        .map(setup::TestAggregateRoot::from)
        .expect("the aggregate root should be found successfully");

    assert_eq!(found_root, root);
}
