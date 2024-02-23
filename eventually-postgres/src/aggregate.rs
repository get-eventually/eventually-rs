//! This module contains the implementation of the [eventually::aggregate::Repository] trait,
//! to work specifically with PostgreSQL databases.
//!
//! Check out the [Repository] type for more information.

use std::marker::PhantomData;

use anyhow::anyhow;
use async_trait::async_trait;
use eventually::aggregate::Aggregate;
use eventually::version::Version;
use eventually::{aggregate, serde, version};
use sqlx::{PgPool, Postgres, Row};

/// Implements the [eventually::aggregate::Repository] trait for
/// PostgreSQL databases.
#[derive(Debug, Clone)]
pub struct Repository<T, Serde, EvtSerde>
where
    T: Aggregate,
    <T as Aggregate>::Id: ToString,
    Serde: serde::Serde<T>,
    EvtSerde: serde::Serde<T::Event>,
{
    pool: PgPool,
    aggregate_serde: Serde,
    event_serde: EvtSerde,
    t: PhantomData<T>,
}

impl<T, Serde, EvtSerde> Repository<T, Serde, EvtSerde>
where
    T: Aggregate,
    <T as Aggregate>::Id: ToString,
    Serde: serde::Serde<T>,
    EvtSerde: serde::Serde<T::Event>,
{
    pub async fn new(
        pool: PgPool,
        aggregate_serde: Serde,
        event_serde: EvtSerde,
    ) -> Result<Self, sqlx::migrate::MigrateError> {
        // Make sure the latest migrations are used before using the Repository instance.
        crate::MIGRATIONS.run(&pool).await?;

        Ok(Self {
            pool,
            aggregate_serde,
            event_serde,
            t: PhantomData,
        })
    }
}

impl<T, Serde, EvtSerde> Repository<T, Serde, EvtSerde>
where
    T: Aggregate + Send + Sync,
    <T as Aggregate>::Id: ToString,
    Serde: serde::Serde<T> + Send + Sync,
    EvtSerde: serde::Serde<T::Event> + Send + Sync,
{
    async fn save_aggregate_state(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        aggregate_id: &str,
        expected_version: Version,
        root: &mut aggregate::Root<T>,
    ) -> Result<(), aggregate::repository::SaveError> {
        let out_state = root.to_aggregate_type::<T>();
        let bytes_state = self
            .aggregate_serde
            .serialize(out_state)
            .map_err(|err| anyhow!("failed to serialize aggregate root state: {}", err))?;

        sqlx::query("CALL upsert_aggregate($1, $2, $3, $4, $5)")
            .bind(aggregate_id)
            .bind(T::type_name())
            .bind(expected_version as i32)
            .bind(root.version() as i32)
            .bind(bytes_state)
            .execute(&mut **tx)
            .await
            .map_err(|err| match crate::check_for_conflict_error(&err) {
                Some(err) => aggregate::repository::SaveError::Conflict(err),
                None => match err.as_database_error().and_then(|err| err.code()) {
                    Some(code) if code == "40001" => version::ConflictError {
                        expected: expected_version,
                        actual: root.version(),
                    }
                    .into(),
                    _ => anyhow!("failed to save aggregate state: {}", err).into(),
                },
            })?;

        Ok(())
    }
}

#[async_trait]
impl<T, Serde, EvtSerde> aggregate::repository::Getter<T> for Repository<T, Serde, EvtSerde>
where
    T: Aggregate + Send + Sync,
    <T as Aggregate>::Id: ToString,
    Serde: serde::Serde<T> + Send + Sync,
    EvtSerde: serde::Serde<T::Event> + Send + Sync,
{
    async fn get(&self, id: &T::Id) -> Result<aggregate::Root<T>, aggregate::repository::GetError> {
        let aggregate_id = id.to_string();

        let row = sqlx::query(
            r#"SELECT version, state
               FROM aggregates
               WHERE aggregate_id = $1 AND "type" = $2"#,
        )
        .bind(&aggregate_id)
        .bind(T::type_name())
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => aggregate::repository::GetError::NotFound,
            _ => anyhow!("failed to fetch the aggregate state row: {}", err).into(),
        })?;

        let version: i32 = row
            .try_get("version")
            .map_err(|err| anyhow!("failed to get 'version' column from row: {}", err))?;

        let bytes_state: Vec<u8> = row
            .try_get("state")
            .map_err(|err| anyhow!("failed to get 'state' column from row: {}", err))?;

        let aggregate: T = self
            .aggregate_serde
            .deserialize(&bytes_state)
            .map_err(|err| {
                anyhow!(
                    "failed to deserialize the aggregate state from the database row: {}",
                    err
                )
            })?;

        Ok(aggregate::Root::rehydrate_from_state(
            version as Version,
            aggregate,
        ))
    }
}

#[async_trait]
impl<T, Serde, EvtSerde> aggregate::repository::Saver<T> for Repository<T, Serde, EvtSerde>
where
    T: Aggregate + Send + Sync,
    <T as Aggregate>::Id: ToString,
    Serde: serde::Serde<T> + Send + Sync,
    EvtSerde: serde::Serde<T::Event> + Send + Sync,
{
    async fn save(
        &self,
        root: &mut aggregate::Root<T>,
    ) -> Result<(), aggregate::repository::SaveError> {
        let events_to_commit = root.take_uncommitted_events();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| anyhow!("failed to begin transaction: {}", err))?;

        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE DEFERRABLE")
            .execute(&mut *tx)
            .await
            .map_err(|err| anyhow!("failed to begin transaction: {}", err))?;

        let aggregate_id = root.aggregate_id().to_string();
        let expected_root_version = root.version() - (events_to_commit.len() as Version);

        self.save_aggregate_state(&mut tx, &aggregate_id, expected_root_version, root)
            .await?;

        crate::event::append_domain_events(
            &mut tx,
            &self.event_serde,
            &aggregate_id,
            root.version() as i32,
            events_to_commit,
        )
        .await
        .map_err(|err| anyhow!("failed to append aggregate root domain events: {}", err))?;

        tx.commit()
            .await
            .map_err(|err| anyhow!("failed to commit transaction: {}", err))?;

        Ok(())
    }
}
