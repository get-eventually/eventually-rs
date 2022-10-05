use std::marker::PhantomData;

use async_trait::async_trait;
use eventually::{
    aggregate,
    aggregate::Aggregate,
    serde::{Deserializer, Serde, Serializer},
    version,
    version::Version,
};
use sqlx::{PgPool, Postgres, Row};

#[derive(Debug, Clone)]
pub struct Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate,
    <T as Aggregate>::Id: ToString,
    OutT: From<T>,
    OutEvt: From<T::Event>,
    TSerde: Serde<OutT>,
    EvtSerde: Serializer<OutEvt>,
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
    OutT: From<T>,
    OutEvt: From<T::Event>,
    TSerde: Serde<OutT>,
    EvtSerde: Serializer<OutEvt>,
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
pub enum GetError {
    #[error("failed to fetch the aggregate state row: {0}")]
    FetchAggregateRow(#[source] sqlx::Error),
    #[error("failed to deserialize the aggregate state from the database row: {0}")]
    DeserializeAggregate(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("failed to convert the aggregate state into its domain type: {0}")]
    ConvertAggregate(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("database returned an error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("failed to begin a new transaction: {0}")]
    BeginTransaction(#[source] sqlx::Error),
    #[error("conflict error detected: {0})")]
    Conflict(#[source] version::ConflictError),
    #[error("concurrent update detected, represented as a conflict error: {0})")]
    Concurrency(#[source] version::ConflictError),
    #[error("failed to save the new aggregate state: {0}")]
    SaveAggregateState(#[source] sqlx::Error),
    #[error("failed to append a new domain event: {0}")]
    AppendEvent(#[source] sqlx::Error),
    #[error("failed to commit transaction: {0}")]
    CommitTransaction(#[source] sqlx::Error),
    #[error("database returned an error: {0}")]
    Database(#[from] sqlx::Error),
}

impl From<SaveError> for Option<version::ConflictError> {
    fn from(err: SaveError) -> Self {
        match err {
            SaveError::Conflict(v) => Some(v),
            SaveError::Concurrency(v) => Some(v),
            _ => None,
        }
    }
}

impl<T, OutT, OutEvt, TSerde, EvtSerde> Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate + Send + Sync,
    <T as Aggregate>::Id: ToString,
    OutT: From<T> + Send + Sync,
    OutEvt: From<T::Event>,
    TSerde: Serde<OutT> + Send + Sync,
    EvtSerde: Serializer<OutEvt>,
{
    async fn save_aggregate_state(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        aggregate_id: &str,
        expected_version: Version,
        root: &mut aggregate::Root<T>,
    ) -> Result<(), SaveError> {
        let out_state = root.to_aggregate_type::<OutT>();
        let bytes_state = self.aggregate_serde.serialize(out_state);

        sqlx::query("CALL upsert_aggregate($1, $2, $3, $4, $5)")
            .bind(aggregate_id)
            .bind(T::type_name())
            .bind(expected_version as i32)
            .bind(root.version() as i32)
            .bind(bytes_state)
            .execute(tx)
            .await
            .map_err(|err| match crate::check_for_conflict_error(&err) {
                Some(err) => SaveError::Conflict(err),
                None => match err.as_database_error().and_then(|err| err.code()) {
                    Some(code) if code == "40001" => {
                        SaveError::Concurrency(version::ConflictError {
                            expected: expected_version,
                            actual: root.version(),
                        })
                    }
                    _ => SaveError::SaveAggregateState(err),
                },
            })?;

        Ok(())
    }
}

#[async_trait]
impl<T, OutT, OutEvt, TSerde, EvtSerde> aggregate::Getter<T>
    for Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate + TryFrom<OutT> + Send + Sync,
    <T as Aggregate>::Id: ToString,
    <T as TryFrom<OutT>>::Error: std::error::Error + Send + Sync + 'static,
    OutT: From<T> + Send + Sync,
    OutEvt: From<T::Event> + Send + Sync,
    TSerde: Serde<OutT> + Send + Sync,
    <TSerde as Deserializer<OutT>>::Error: std::error::Error + Send + Sync + 'static,
    EvtSerde: Serializer<OutEvt> + Send + Sync,
{
    type Error = GetError;

    async fn get(
        &self,
        id: &T::Id,
    ) -> Result<aggregate::Root<T>, aggregate::RepositoryGetError<Self::Error>> {
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
            sqlx::Error::RowNotFound => aggregate::RepositoryGetError::AggregateRootNotFound,
            _ => aggregate::RepositoryGetError::Inner(GetError::FetchAggregateRow(err)),
        })?;

        let version: i32 = row.try_get("version").map_err(GetError::Database)?;
        let bytes_state: Vec<u8> = row.try_get("state").map_err(GetError::Database)?;

        let aggregate: T = self
            .aggregate_serde
            .deserialize(bytes_state)
            .map_err(|err| GetError::DeserializeAggregate(Box::new(err)))
            .and_then(|out_t| {
                T::try_from(out_t).map_err(|err| GetError::ConvertAggregate(Box::new(err)))
            })?;

        Ok(aggregate::Root::rehydrate_from_state(
            version as Version,
            aggregate,
        ))
    }
}

#[async_trait]
impl<T, OutT, OutEvt, TSerde, EvtSerde> aggregate::Saver<T>
    for Repository<T, OutT, OutEvt, TSerde, EvtSerde>
where
    T: Aggregate + TryFrom<OutT> + Send + Sync,
    <T as Aggregate>::Id: ToString,
    <T as TryFrom<OutT>>::Error: std::error::Error + Send + Sync + 'static,
    OutT: From<T> + Send + Sync,
    OutEvt: From<T::Event> + Send + Sync,
    TSerde: Serde<OutT> + Send + Sync,
    <TSerde as Deserializer<OutT>>::Error: std::error::Error + Send + Sync + 'static,
    EvtSerde: Serializer<OutEvt> + Send + Sync,
{
    type Error = SaveError;

    async fn store(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::Error> {
        let events_to_commit = root.take_uncommitted_events();

        if events_to_commit.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(SaveError::BeginTransaction)?;

        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE DEFERRABLE")
            .execute(&mut tx)
            .await?;

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
        .map_err(SaveError::AppendEvent)?;

        tx.commit().await.map_err(SaveError::CommitTransaction)?;

        Ok(())
    }
}
