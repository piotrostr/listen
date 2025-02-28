#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().ok();
    }
    listen_engine::metrics::init_metrics();

    tracing::info!("Starting listen-engine...");

    listen_engine::server::run().await
}
