use eventually::{aggregate::Repository, serde::json::Json};
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

    let mut root = setup::TestAggregateRoot::create(aggregate_id, "John Dee".to_owned())
        .expect("aggregate root should be created");

    aggregate_repository
        .store(&mut root)
        .await
        .expect("no failure");
}
