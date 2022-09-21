use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use eventually::{
    aggregate,
    aggregate::{repository, Aggregate},
    version,
};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct AggregateRepository<T, R, OutT, OutEvt> {
    pool: PgPool,
    aggregate_root_type_name: Arc<String>,
    aggregate_type: PhantomData<T>,
    aggregate_root_type: PhantomData<R>,
    out_aggregate_type: PhantomData<OutT>,
    out_event_type: PhantomData<OutEvt>,
}

impl<T, R, OutT, OutEvt> AggregateRepository<T, R, OutT, OutEvt>
where
    T: Aggregate,
    R: aggregate::Root<T>,
{
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            aggregate_root_type_name: Arc::new(R::type_name().to_owned()),
            aggregate_type: PhantomData,
            aggregate_root_type: PhantomData,
            out_aggregate_type: PhantomData,
            out_event_type: PhantomData,
        }
    }

    pub fn with_aggregate_root_type_name(mut self, name: &str) -> Self {
        self.aggregate_root_type_name = Arc::new(name.to_owned());
        self
    }
}

#[async_trait]
impl<T, R, OutT, OutEvt> aggregate::Repository<T, R> for AggregateRepository<T, R, OutT, OutEvt>
where
    T: Aggregate,
    R: aggregate::Root<T>,
    for<'a> OutT: From<T> + Into<T> + TryFrom<&'a [u8]> + Into<Vec<u8>> + Send + Sync,
    OutEvt: From<T::Event> + Into<Vec<u8>> + Send + Sync,
    // TODO: remove this
    for<'a> <OutT as TryFrom<&'a [u8]>>::Error: std::fmt::Debug,
    <T as TryFrom<OutT>>::Error: std::fmt::Debug,
{
    type Error = sqlx::Error;

    async fn get(&self, id: &T::Id) -> Result<R, repository::GetError<Self::Error>> {
        let query = sqlx::query(
            r#"SELECT version, state
		    FROM aggregates
		    WHERE aggregate_id = $1 AND "type" = $2"#,
        );

        let result = query
            .bind(id.to_string())
            .bind(self.aggregate_root_type_name.as_ref())
            .fetch_one(&self.pool)
            .await;

        if let Err(sqlx::Error::RowNotFound) = result {
            return Err(repository::GetError::AggregateRootNotFound);
        }

        let row = result?;
        let version_column: i64 = row.try_get("version")?;
        let state_column: Vec<u8> = row.try_get("state")?;

        let deserialized_state = OutT::try_from(&state_column).unwrap();
        let aggregate_root_state: T = deserialized_state.into();

        Ok(R::from(aggregate::Context::from_state(
            version_column as version::Version,
            aggregate_root_state,
        )))
    }

    async fn store(&self, _root: &mut R) -> Result<(), Self::Error> {
        todo!()
    }
}
