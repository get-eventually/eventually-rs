use std::marker::PhantomData;

use async_trait::async_trait;
use eventually::{aggregate, aggregate::Aggregate, serde::Serde};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Repository<T, OutT, OutEvt, TSerde, EvtSerde> {
    pool: PgPool,
    t: PhantomData<T>,
    out_t: PhantomData<OutT>,
    out_evt: PhantomData<OutEvt>,
    evt_serde: PhantomData<EvtSerde>,
    t_serde: PhantomData<TSerde>,
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
    type Error = ();

    async fn get(&self, id: &T::Id) -> Result<aggregate::Root<T>, Self::Error> {
        todo!()
    }

    async fn store(&self, root: &mut aggregate::Root<T>) -> Result<(), Self::Error> {
        todo!()
    }
}
