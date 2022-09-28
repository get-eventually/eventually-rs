use std::marker::PhantomData;

use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Repository<T, EvtSerde, TSerde> {
    pool: PgPool,
    t: PhantomData<T>,
    evt_serde: PhantomData<EvtSerde>,
    t_serde: PhantomData<TSerde>,
}
