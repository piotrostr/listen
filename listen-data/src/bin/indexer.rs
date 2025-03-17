use anyhow::Result;
use clap::Parser;
use listen_data::{
    geyser::make_geyser_pipeline,
    metrics::SwapMetrics,
    sol_price_stream::SolPriceCache,
    util::{make_db, make_kv_store, make_message_queue},
};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Parser)]
pub struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    listen_tracing::setup_tracing();
    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().expect("Failed to load .env file");
    }
    info!("Starting geyser indexer...");

    let db = make_db().await?;
    let kv_store = make_kv_store().await?;
    let message_queue = make_message_queue().await?;
    let swap_metrics = Arc::new(SwapMetrics::new());
    let price_cache =
        SolPriceCache::new(Some(kv_store.clone()), Some(message_queue.clone()));
    let price_cache = Arc::new(price_cache);

    info!("Solana price: {}", price_cache.get_price().await);

    let mut pipeline =
        make_geyser_pipeline(kv_store, message_queue, db, swap_metrics)?;

    tokio::spawn(async move {
        if let Err(e) = price_cache.start_price_stream().await {
            error!("Error in SOL price stream: {}", e);
        }
    });

    pipeline.run().await?;

    Ok(())
}
