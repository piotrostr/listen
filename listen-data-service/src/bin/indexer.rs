use anyhow::Result;
use clap::Parser;
use listen_data_service::{
    geyser::make_raydium_geyser_instruction_pipeline,
    sol_price_stream::SOL_PRICE_CACHE,
    util::{make_db, make_kv_store, make_message_queue},
};
use tracing::{error, info};

#[derive(Parser)]
pub struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenv::dotenv().expect("Failed to load .env file");
    info!("Starting geyser indexer...");

    // Initialize price cache for cold starts
    info!("Solana price: {}", SOL_PRICE_CACHE.get_price().await);

    let db = make_db().await?;
    let kv_store = make_kv_store()?;
    let message_queue = make_message_queue()?;

    let mut pipeline =
        make_raydium_geyser_instruction_pipeline(kv_store, message_queue, db)?;

    let price_cache = SOL_PRICE_CACHE.clone();

    tokio::spawn(async move {
        if let Err(e) = price_cache.start_price_stream().await {
            error!("Error in SOL price stream: {}", e);
        }
    });

    pipeline.run().await?;

    Ok(())
}
