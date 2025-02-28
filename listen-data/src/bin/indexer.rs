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

pub fn setup_tracing() {
    use tracing_subscriber::{
        filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt,
    };
    // Create an EnvFilter that reads from RUST_LOG with INFO as default
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Configure logging based on environment
    if std::env::var("IS_SYSTEMD_SERVICE").is_ok() {
        // Use systemd formatting when running as a service
        let journald_layer =
            tracing_journald::layer().expect("Failed to create journald layer");
        tracing_subscriber::registry()
            .with(journald_layer)
            .with(env_filter)
            .init();
    } else {
        // Use standard formatting for non-systemd environments
        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_target(true)
            .with_env_filter(env_filter)
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    if !std::env::var("IS_SYSTEMD_SERVICE").is_ok() {
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
