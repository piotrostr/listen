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
}
