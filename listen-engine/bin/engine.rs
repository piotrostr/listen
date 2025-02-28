use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Replace the basic init with a custom configuration
    let is_systemd = std::env::var("IS_SYSTEMD_SERVICE").is_ok();

    if is_systemd {
        // Use systemd formatting when running as a service
        let journald_layer = tracing_journald::layer().unwrap();
        tracing_subscriber::registry().with(journald_layer).init();
    } else {
        // Use standard formatting for non-systemd environments
        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_target(true)
            .init();
    }

    if !is_systemd {
        dotenv::dotenv().ok();
    }
    listen_engine::metrics::init_metrics();

    tracing::info!("Starting listen-engine...");

    listen_engine::server::run().await
}
