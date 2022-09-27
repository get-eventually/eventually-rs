use std::time::Duration;

use anyhow::anyhow;
use eventually::serde::prost::MessageSerde;
use eventually_postgres::store::EventStore;

use bank_accounting::{application, domain::BankAccountRepository, grpc, proto};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize stdout logger for the application.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .map_err(|e| anyhow!("failed to initialize tracing logger: {}", e))?;

    let database_url = std::env::var("DATABASE_URL").expect("env var DATABASE_URL is required");
    let pool = sqlx::PgPool::connect(&database_url).await?;

    let bank_account_event_serde = MessageSerde::<proto::Event>::default();
    let bank_account_event_store = EventStore::new(pool, bank_account_event_serde).await?;
    let bank_account_repository = BankAccountRepository::from(bank_account_event_store.clone());

    let application_service = application::Service::from(bank_account_repository);

    tracing::info!("Service is starting up...");

    let addr = "0.0.0.0:10437"
        .parse()
        .map_err(|e| anyhow!("failed to parse grpc address: {}", e))?;

    let (_, health_svc) = tonic_health::server::health_reporter();

    let reflection_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(
            tonic_health::proto::GRPC_HEALTH_V1_FILE_DESCRIPTOR_SET,
        )
        .build()
        .map_err(|e| anyhow!("failed to build grpc reflection service: {}", e))?;

    let bank_accounting_svc = proto::bank_accounting_server::BankAccountingServer::new(
        grpc::BankAccountingApi::from(application_service),
    );

    let layer = tower::ServiceBuilder::new()
        .timeout(Duration::from_secs(5))
        .into_inner();

    tonic::transport::Server::builder()
        .layer(layer)
        .add_service(health_svc)
        .add_service(reflection_svc)
        .add_service(bank_accounting_svc)
        .serve(addr)
        .await
        .map_err(|e| anyhow!("tonic server exited with error: {}", e))?;

    Ok(())
}
