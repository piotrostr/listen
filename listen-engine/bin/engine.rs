use listen_tracing::setup_tracing;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    setup_tracing();

    tracing::info!("Starting listen-engine...");

    listen_engine::server::run().await
}
