use anyhow::Result;
use clap::Parser;
use listen_data::{
    geyser::make_raydium_geyser_instruction_pipeline,
    sol_price_stream::SolPriceCache,
    util::{make_db, make_kv_store, make_message_queue},
};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Parser)]
pub struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().expect("Failed to load .env file");
    }
    info!("Starting geyser indexer...");

    let db = make_db().await?;
    let kv_store = make_kv_store().await?;
    let message_queue = make_message_queue().await?;
    let price_cache =
        SolPriceCache::new(Some(kv_store.clone()), Some(message_queue.clone()));
    let price_cache = Arc::new(price_cache);

    info!("Solana price: {}", price_cache.get_price().await);

    let mut pipeline =
        make_raydium_geyser_instruction_pipeline(kv_store, message_queue, db)?;

    tokio::spawn(async move {
        if let Err(e) = price_cache.start_price_stream().await {
            error!("Error in SOL price stream: {}", e);
        }
    });

    pipeline.run().await?;

    Ok(())
}
