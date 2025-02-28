use actix_web::{HttpResponse, Responder};
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;

static PROMETHEUS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Failed to install metrics recorder")]
    InstallRecorderError(BuildError),
}

pub fn setup_metrics_exporter() -> Result<PrometheusHandle, MetricsError> {
    // Create Prometheus recorder
    let builder = PrometheusBuilder::new();

    // Add global labels if needed
    let builder = builder.add_global_label("service", "listen-engine");

    // Install global recorder
    let handle = builder
        .install_recorder()
        .map_err(MetricsError::InstallRecorderError)?;

    // Store handle in global state
    let _ = PROMETHEUS_HANDLE.set(handle.clone());

    Ok(handle)
}

// Metrics endpoint handler for actix-web
pub async fn metrics_handler() -> impl Responder {
    let handle = PROMETHEUS_HANDLE
        .get()
        .expect("Prometheus handle not initialized");
    let metrics = handle.render();

    HttpResponse::Ok().content_type("text/plain").body(metrics)
}

pub fn init_metrics() {
    let _handle = setup_metrics_exporter().expect("Failed to setup metrics exporter");

    metrics::describe_counter!(
        "price_updates_processed",
        "Number of price updates processed"
    );
    metrics::describe_histogram!(
        "price_update_duration",
        "Time taken to process price updates"
    );
    metrics::describe_counter!("pipeline_evaluations", "Number of pipeline evaluations");
    metrics::describe_histogram!(
        "pipeline_evaluation_duration",
        "Time taken to evaluate pipelines"
    );
    metrics::describe_gauge!("active_pipelines", "Number of active pipelines");
    metrics::describe_gauge!(
        "redis_subscriber_healthy",
        "Whether the Redis subscriber is running"
    );
    metrics::describe_counter!(
        "redis_reconnection_attempts",
        "Number of times the Redis subscriber attempted to reconnect"
    );
    metrics::describe_counter!(
        "price_updates_received",
        "Number of price updates received from Redis"
    );
    metrics::describe_counter!(
        "price_updates_parse_errors",
        "Number of price update parsing errors"
    );

    // New detailed Redis subscriber metrics
    metrics::describe_counter!(
        "redis_messages_received",
        "Number of raw messages received from Redis"
    );
    metrics::describe_counter!(
        "price_updates_parsed",
        "Number of price updates successfully parsed"
    );
    metrics::describe_counter!(
        "price_updates_sent",
        "Number of price updates sent to channel"
    );
    metrics::describe_counter!(
        "price_update_channel_full",
        "Number of times the price update channel was full"
    );
    metrics::describe_counter!(
        "price_updates_send_errors",
        "Number of errors sending price updates"
    );
    metrics::describe_counter!(
        "redis_payload_errors",
        "Number of Redis payload parsing errors"
    );
    metrics::describe_counter!(
        "redis_subscriber_exits",
        "Number of times the Redis subscriber task has exited"
    );
    metrics::describe_gauge!(
        "redis_subscriber_last_message_age_seconds",
        "Seconds since last message was received"
    );
    metrics::describe_gauge!(
        "price_update_channel_capacity",
        "Available capacity in the price update channel"
    );
}
