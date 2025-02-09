use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use clap::Parser;
use listen_data_service::{
    account_pipeline::{make_jupiter_accounts_pipeline, make_raydium_accounts_pipeline},
    instruction_pipeline::make_raydium_instruction_pipeline,
    sol_price_stream::SolPriceCache,
};
use tracing::{error, info};

#[derive(Parser)]
pub enum Command {
    RaydiumAccounts,
    JupiterAccounts,
    RaydiumInstructions,
}

#[tokio::main]
async fn main() -> Result<()> {
    let command = Command::parse();
    tracing_subscriber::fmt().init();
    info!("Starting up...");

    let mut pipeline: Pipeline;
    match command {
        Command::RaydiumAccounts => {
            pipeline = make_raydium_accounts_pipeline()?;
        }
        Command::JupiterAccounts => {
            pipeline = make_jupiter_accounts_pipeline()?;
        }
        Command::RaydiumInstructions => {
            pipeline = make_raydium_instruction_pipeline()?;
        }
    }

    let price_cache = SolPriceCache::new();

    tokio::spawn(async move {
        if let Err(e) = price_cache.start_price_stream().await {
            error!("Error in SOL price stream: {}", e);
        }
    });

    pipeline.run().await?;

    Ok(())
}
