use log::info;
use prometheus::{Encoder, IntCounter, Registry, TextEncoder};
use std::sync::Arc;
use warp::Filter;

static TRANSACTIONS_RECEIVED: &str = "transactions_received";
static TRANSACTIONS_PROCESSED: &str = "transactions_processed";
static REQUESTS_SENT: &str = "requests_sent";

pub fn setup_metrics(
) -> (Arc<IntCounter>, Arc<IntCounter>, Arc<IntCounter>, Registry) {
    let registry = Registry::new();
    let transactions_received = IntCounter::new(
        TRANSACTIONS_RECEIVED,
        "Total number of transactions received",
    )
    .unwrap();
    let transactions_processed = IntCounter::new(
        TRANSACTIONS_PROCESSED,
        "Total number of transactions processed",
    )
    .unwrap();

    let requests_sent = IntCounter::new(
        REQUESTS_SENT,
        "Total number of requests sent to the server",
    )
    .unwrap();

    registry
        .register(Box::new(transactions_received.clone()))
        .unwrap();
    registry
        .register(Box::new(transactions_processed.clone()))
        .unwrap();
    registry.register(Box::new(requests_sent.clone())).unwrap();

    (
        Arc::new(transactions_received),
        Arc::new(transactions_processed),
        Arc::new(requests_sent),
        registry,
    )
}

pub async fn run_metrics_server(registry: Registry) {
    // Metrics endpoint
    let metrics_route = warp::path!("metrics").map(move || {
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        warp::reply::with_header(buffer, "Content-Type", encoder.format_type())
    });

    info!("Metrics server running on {}", 3030);
    warp::serve(metrics_route).run(([127, 0, 0, 1], 3030)).await;
}
