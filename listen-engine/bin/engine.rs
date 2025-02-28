#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Import the Layer trait to use the `with` method
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Replace the simple init with a more configurable setup
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::env::var("NO_COLOR").is_err())
        .with_writer(std::io::stderr);

    // Use JSON format if specified in environment
    if std::env::var("RUST_LOG_FORMAT").unwrap_or_default() == "json" {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().json())
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().ok();
    }
    listen_engine::metrics::init_metrics();

    tracing::info!("Starting listen-engine...");

    listen_engine::server::run().await
}
