use anyhow::anyhow;
use tracing_subscriber::{prelude::*, EnvFilter};

pub fn initialize(service_name: &str) -> anyhow::Result<()> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(service_name)
        .install_simple()
        .map_err(|e| anyhow!("failed to initialize jaeger tracer: {}", e))?;

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| anyhow!("failed to initialize env filter: {}", e))?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(filter_layer)
        .try_init()
        .map_err(|e| anyhow!("failed to initialize subscribers: {}", e))?;

    Ok(())
}
