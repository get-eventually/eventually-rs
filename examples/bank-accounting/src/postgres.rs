use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::{ConnectOptions, PgPool};
use tracing::log::LevelFilter;

pub async fn connect() -> anyhow::Result<PgPool> {
    let mut connect_options = PgConnectOptions::new()
        .host(
            std::env::var("DATABASE_HOST")
                .expect("env var DATABASE_HOST is required")
                .as_ref(),
        )
        .port(5432)
        .username("postgres")
        .password("password")
        .ssl_mode(PgSslMode::Disable)
        .log_statements(LevelFilter::Debug);

    Ok(PgPool::connect_with(connect_options).await?)
}
