#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Replace the basic init with a custom configuration
    let is_systemd = std::env::var("IS_SYSTEMD_SERVICE").is_ok();

    // Configure tracing with ANSI colors disabled for systemd
    tracing_subscriber::fmt()
        .with_ansi(!is_systemd) // Disable ANSI colors when running as systemd service
        .with_target(true)
        .init();

    if !is_systemd {
        dotenv::dotenv().ok();
    }
    listen_engine::metrics::init_metrics();

    tracing::info!("Starting listen-engine...");

    listen_engine::server::run().await
}
