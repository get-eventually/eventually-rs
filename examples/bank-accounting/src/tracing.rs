use anyhow::anyhow;
use opentelemetry::KeyValue;
use opentelemetry_sdk::{trace, Resource};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub fn initialize(service_name: &'static str) -> anyhow::Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(trace::Sampler::AlwaysOn)
                .with_id_generator(trace::RandomIdGenerator::default())
                .with_resource(Resource::new([KeyValue::new("service.name", service_name)])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| anyhow!("failed to initialize OTLP tracer: {}", e))?;

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|e| anyhow!("failed to initialize env filter: {}", e))?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(filter_layer)
        .try_init()
        .map_err(|e| anyhow!("failed to initialize subscribers: {}", e))?;

    Ok(())
}
