use eventually::aggregate::repository::{self, GetError, Getter, Saver};
use eventually::serde;
use eventually_postgres::aggregate;
use rand::Rng;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

mod setup;

#[tokio::test]
async fn it_works() {
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

    let aggregate_repository = aggregate::Repository::new(
        pool,
        serde::Json::<setup::TestAggregate>::default(),
        serde::Json::<setup::TestDomainEvent>::default(),
    )
    .await
    .unwrap();

    let aggregate_id = setup::TestAggregateId(rand::rng().random::<i64>());

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

    let aggregate_repository = aggregate::Repository::new(
        pool,
        serde::Json::<setup::TestAggregate>::default(),
        serde::Json::<setup::TestDomainEvent>::default(),
    )
    .await
    .unwrap();

    let aggregate_id = setup::TestAggregateId(rand::rng().random::<i64>());

    let mut root = setup::TestAggregateRoot::create(aggregate_id, "John Dee".to_owned())
        .expect("aggregate root should be created");

    // We also delete it just to cause more Domain Events in its Event Stream.
    root.delete().unwrap();

    // We clone the Aggregate Root instance so that we have the same
    // uncommitted events list as the original instance.
    let mut cloned_root = root.clone();

    let result = futures::join!(
        aggregate_repository.save(&mut root),
        aggregate_repository.save(&mut cloned_root),
    );

    match result {
        (Ok(()), Err(repository::SaveError::Conflict(_))) => (),
        (Err(repository::SaveError::Conflict(_)), Ok(())) => (),
        (first, second) => panic!(
            "invalid state detected, first: {:?}, second: {:?}",
            first, second
        ),
    };
}
