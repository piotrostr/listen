use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use clap::Parser;
use listen_data_service::{db::Database, sol_price_stream::SOL_PRICE_CACHE};

#[cfg(feature = "geyser")]
use listen_data_service::geyser::make_raydium_geyser_instruction_pipeline;

use listen_data_service::db::ClickhouseDb;
#[cfg(feature = "rpc")]
use listen_data_service::rpc::{
    account_pipeline::make_raydium_rpc_accounts_pipeline,
    instruction_pipeline::make_raydium_rpc_instruction_pipeline,
};
use tracing::{error, info};

#[cfg(feature = "rpc")]
#[derive(Parser)]
pub enum Command {
    RaydiumAccountsRpc,
    RaydiumInstructionsRpc,
}

#[cfg(feature = "geyser")]
#[derive(Parser)]
pub enum Command {
    RaydiumInstructionsGeyser,
}

#[cfg(not(any(feature = "rpc", feature = "geyser")))]
#[derive(Parser)]
pub struct Command {}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenv::dotenv().expect("Failed to load .env file");
    info!("Starting up...");

    #[cfg(not(any(feature = "rpc", feature = "geyser")))]
    {
        error!("Error: No features enabled. Please enable either 'rpc' or 'geyser' feature.");
        error!("Example: cargo run --feature rpc");
        std::process::exit(1);
    }

    #[cfg(any(feature = "rpc", feature = "geyser"))]
    {
        let command = Command::parse();

        // be sure to call this
        ClickhouseDb::new().initialize().await?;

        let mut pipeline: Pipeline;
        #[cfg(feature = "rpc")]
        match command {
            Command::RaydiumAccountsRpc => {
                pipeline = make_raydium_rpc_accounts_pipeline()?;
            }
            Command::RaydiumInstructionsRpc => {
                pipeline = make_raydium_rpc_instruction_pipeline()?;
            }
        }

        #[cfg(feature = "geyser")]
        match command {
            Command::RaydiumInstructionsGeyser => {
                pipeline = make_raydium_geyser_instruction_pipeline()?;
            }
        }

        let price_cache = SOL_PRICE_CACHE.clone();

        tokio::spawn(async move {
            if let Err(e) = price_cache.start_price_stream().await {
                error!("Error in SOL price stream: {}", e);
            }
        });

        pipeline.run().await?;
    }

    Ok(())
}
