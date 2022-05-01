use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use eventually::{aggregate, aggregate::Aggregate, version};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct AggregateRepository<T, R, OutT, OutEvt>
where
    T: Aggregate + TryFrom<OutT>,
    R: aggregate::Root<T>,
    OutT: From<T> + TryFrom<Vec<u8>> + Into<Vec<u8>>,
    OutEvt: From<T::Event> + Into<Vec<u8>>,
{
    pool: PgPool,
    aggregate_root_type_name: Arc<String>,
    aggregate_type: PhantomData<T>,
    aggregate_root_type: PhantomData<R>,
    out_aggregate_type: PhantomData<OutT>,
    out_event_type: PhantomData<OutEvt>,
}

impl<T, R, OutT, OutEvt> AggregateRepository<T, R, OutT, OutEvt>
where
    T: Aggregate + TryFrom<OutT>,
    R: aggregate::Root<T>,
    OutT: From<T> + TryFrom<Vec<u8>> + Into<Vec<u8>>,
    OutEvt: From<T::Event> + Into<Vec<u8>>,
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
    T: Aggregate + TryFrom<OutT> + TryFrom<Vec<u8>> + Into<Vec<u8>>,
    T::Event: Into<Vec<u8>>,
    R: aggregate::Root<T>,
    OutT: From<T> + TryFrom<Vec<u8>> + Into<Vec<u8>> + Send + Sync,
    OutEvt: From<T::Event> + Into<Vec<u8>> + Send + Sync,
    // TODO: remove this
    <OutT as TryFrom<Vec<u8>>>::Error: std::fmt::Debug,
    <T as TryFrom<OutT>>::Error: std::fmt::Debug,
{
    type Error = sqlx::Error;

    async fn get(&self, id: &T::Id) -> Result<R, aggregate::RepositoryGetError<Self::Error>> {
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
            return Err(aggregate::RepositoryGetError::AggregateRootNotFound);
        }

        let row = result?;
        let version_column: i64 = row.try_get("version")?;
        let state_column: Vec<u8> = row.try_get("state")?;

        let deserialized_state = OutT::try_from(state_column).unwrap();
        let aggregate_root_state = T::try_from(deserialized_state).unwrap();

        Ok(R::from(aggregate::Context::from_state(
            version_column as version::Version,
            aggregate_root_state,
        )))
    }

    async fn store(&self, root: &mut R) -> Result<(), Self::Error> {
        todo!()
    }
}
