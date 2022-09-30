use std::marker::PhantomData;

use async_trait::async_trait;
use eventually::{aggregate, aggregate::Aggregate, serde::Serde, version::Version};
use sqlx::{PgPool, Postgres};

#[derive(Debug, Clone)]
pub struct Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate,
    <T as Aggregate>::Id: ToString,
    for<'a> OutT: From<&'a T>,
    OutEvt: From<T::Event>,
    TSerde: Serde<OutT>,
    EvtSerde: Serde<OutEvt>,
{
    pool: PgPool,
    aggregate_serde: TSerde,
    event_serde: EvtSerde,
    t: PhantomData<T>,
    out_t: PhantomData<OutT>,
    out_evt: PhantomData<OutEvt>,
}

impl<T, OutT, OutEvt, TSerde, EvtSerde> Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate,
    <T as Aggregate>::Id: ToString,
    for<'a> OutT: From<&'a T>,
    OutEvt: From<T::Event>,
    TSerde: Serde<OutT>,
    EvtSerde: Serde<OutEvt>,
{
    pub async fn new(
        pool: PgPool,
        aggregate_serde: TSerde,
        event_serde: EvtSerde,
    ) -> Result<Self, sqlx::migrate::MigrateError> {
        // Make sure the latest migrations are used before using the Repository instance.
        crate::MIGRATIONS.run(&pool).await?;

        Ok(Self {
            pool,
            aggregate_serde,
            event_serde,
            t: PhantomData,
            out_t: PhantomData,
            out_evt: PhantomData,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("failed to begin a new transaction: {0}")]
    BeginTransaction(#[source] sqlx::Error),
    #[error("failed to save the new aggregate state: {0}")]
    SaveAggregateState(#[source] sqlx::Error),
    #[error("failed to commit transaction: {0}")]
    CommitTransaction(#[source] sqlx::Error),
    #[error("database returned an error: {0}")]
    Database(#[from] sqlx::Error),
}

impl<T, OutT, OutEvt, TSerde, EvtSerde> Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate + Send + Sync,
    <T as Aggregate>::Id: ToString,
    for<'a> OutT: From<&'a T> + Send + Sync,
    OutEvt: From<T::Event>,
    TSerde: Serde<OutT> + Send + Sync,
    EvtSerde: Serde<OutEvt>,
{
    async fn save_aggregate_state(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        expected_version: Version,
        root: &mut aggregate::Root<T>,
    ) -> Result<(), RepositoryError> {
        let event_stream_id = root.aggregate_id().to_string();

        let out_state = root.to_aggregate_type::<OutT>();
        let bytes_state = self.aggregate_serde.serialize(out_state);

        sqlx::query("CALL upsert_aggregate($1, $2, $3, $4, $5)")
            .bind(&event_stream_id)
            .bind("test-type-please-change")
            .bind(expected_version as i32)
            .bind(root.version() as i32)
            .bind(bytes_state)
            .execute(tx)
            .await
            .map_err(RepositoryError::SaveAggregateState)?;

        Ok(())
    }
}

#[async_trait]
impl<T, OutT, OutEvt, TSerde, EvtSerde> aggregate::Repository<T>
    for Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate + TryFrom<OutT> + Send + Sync,
    <T as Aggregate>::Id: ToString,
    for<'a> OutT: From<&'a T> + Send + Sync,
    OutEvt: From<T::Event> + Send + Sync,
    TSerde: Serde<OutT> + Send + Sync,
    EvtSerde: Serde<OutEvt> + Send + Sync,
{
    type Error = RepositoryError;

    async fn get(&self, id: &T::Id) -> Result<aggregate::Root<T>, Self::Error> {
        todo!()
    }

    async fn store(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::Error> {
        let events_to_commit = root.take_uncommitted_events();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(RepositoryError::BeginTransaction)?;

        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE DEFERRABLE")
            .execute(&mut tx)
            .await?;

        let expected_root_version = root.version() - (events_to_commit.len() as Version);

        self.save_aggregate_state(&mut tx, expected_root_version, root)
            .await?;

        tx.commit()
            .await
            .map_err(RepositoryError::CommitTransaction)?;

        Ok(())
    }
}
