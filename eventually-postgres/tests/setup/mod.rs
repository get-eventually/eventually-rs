use std::time::Instant;

use eventually::message::Message;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub async fn connect_to_database() -> Result<PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("the env var DATABASE_URL is required");

    sqlx::PgPool::connect(&url).await
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestDomainEvent {
    WasCreated { name: String, at: u128 },
    WasDeleted { name: String },
}

impl Message for TestDomainEvent {
    fn name(&self) -> &'static str {
        match self {
            TestDomainEvent::WasCreated { .. } => "TestDomainSomethingWasCreated",
            TestDomainEvent::WasDeleted { .. } => "TestDomainSomethingWasDeleted",
        }
    }
}
