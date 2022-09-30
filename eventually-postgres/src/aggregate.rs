use std::marker::PhantomData;

use async_trait::async_trait;
use eventually::{aggregate, aggregate::Aggregate, serde::Serde, version::Version};
use sqlx::{PgPool, Postgres};

#[derive(Debug, Clone)]
pub struct Repository<T, OutT, OutEvt, TSerde, EvtSerde> {
    pool: PgPool,
    aggregate_serde: TSerde,
    t: PhantomData<T>,
    out_t: PhantomData<OutT>,
    out_evt: PhantomData<OutEvt>,
    evt_serde: PhantomData<EvtSerde>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("failed to begin a new transaction: {0}")]
    BeginTransaction(#[source] sqlx::Error),
    #[error("database returned an error: {0}")]
    Database(#[from] sqlx::Error),
}

impl<T, OutT, OutEvt, TSerde, EvtSerde> Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate + Send + Sync,
    for<'a> OutT: From<&'a T> + Send + Sync,
    TSerde: Serde<OutT> + Send + Sync,
{
    async fn save_aggregate_state(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        expected_version: Version,
        root: aggregate::Root<T>,
    ) -> Result<(), RepositoryError> {
        let out_state = root.to_aggregate_type::<OutT>();
        let bytes_state = self.aggregate_serde.serialize(out_state);

        todo!()
    }
}

#[async_trait]
impl<T, OutT, OutEvt, TSerde, EvtSerde> aggregate::Repository<T>
    for Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate + TryFrom<OutT> + Send + Sync,
    OutT: From<T> + Send + Sync,
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

        todo!()
    }
}
