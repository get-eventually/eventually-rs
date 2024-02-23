use std::time::Duration;

use anyhow::anyhow;
use bank_accounting::domain::{BankAccountEvent, BankAccountRepository};
use bank_accounting::{application, grpc, proto};
use eventually::serde;
use eventually::tracing::{AggregateRepositoryExt, EventStoreExt};
use eventually_postgres::event;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bank_accounting::tracing::initialize("bank-accounting")?;

    let pool = bank_accounting::postgres::connect().await?;

    let bank_account_event_serde = serde::Convert::<BankAccountEvent, proto::Event, _>::new(
        serde::Protobuf::<proto::Event>::default(),
    );

    let bank_account_event_store = event::Store::new(pool, bank_account_event_serde)
        .await?
        .with_tracing();

    let bank_account_repository =
        BankAccountRepository::from(bank_account_event_store.clone()).with_tracing();

    let application_service = application::Service::from(bank_account_repository);

    tracing::info!("Service is starting up...");

    let addr = "0.0.0.0:10437"
        .parse()
        .map_err(|e| anyhow!("failed to parse grpc address: {}", e))?;

    let (_, health_svc) = tonic_health::server::health_reporter();

    let reflection_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build()
        .map_err(|e| anyhow!("failed to build grpc reflection service: {}", e))?;

    let bank_accounting_svc = proto::bank_accounting_server::BankAccountingServer::new(
        grpc::BankAccountingApi::from(application_service),
    );

    let layer = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
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
