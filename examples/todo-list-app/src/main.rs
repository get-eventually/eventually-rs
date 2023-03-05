use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use eventually::{
    aggregate, event,
    tracing::{AggregateRepositoryExt, EventStoreExt},
};
use tower_http::trace::TraceLayer;

use todo_list_app::{command, grpc, proto};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    todo_list_app::tracing::initialize("todo-list-app")?;

    let event_store = event::store::InMemory::default().with_tracing();
    let event_sourced_repository =
        aggregate::EventSourcedRepository::from(event_store).with_tracing();

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

    let bank_accounting_svc =
        proto::todo_list_service_server::TodoListServiceServer::new(grpc::TodoListService {
            id_generator: Arc::new(|| Uuid::new_v4().to_string()),
            create_todo_list: command::create_todo_list::Handler::new(
                Instant::now,
                event_sourced_repository.clone(),
            ),
            add_todo_list_item: command::add_todo_list_item::Handler::new(
                Instant::now,
                event_sourced_repository,
            ),
        });

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
