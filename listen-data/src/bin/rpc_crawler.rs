use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub enum Command {
    RaydiumAccountsRpc,
    RaydiumInstructionsRpc,
}

#[cfg(feature = "rpc")]
#[tokio::main]
async fn main() -> Result<()> {
    use listen_data::{
        rpc::{
            account_pipeline::make_raydium_rpc_accounts_pipeline,
            instruction_pipeline::make_raydium_rpc_instruction_pipeline,
        },
        sol_price_stream::SOL_PRICE_CACHE,
        util::{make_db, make_kv_store, make_message_queue},
    };
    use tracing::{error, info};

    tracing_subscriber::fmt().init();
    if std::env::var("IS_SYSTEMD_SERVICE").is_err() {
        dotenv::dotenv().expect("Failed to load .env file");
    }
    info!("Starting RPC service...");

    // Initialize price cache for cold starts
    info!("Solana price: {}", SOL_PRICE_CACHE.get_price().await);

    let db = make_db().await?;
    let kv_store = make_kv_store().await?;
    let message_queue = make_message_queue().await?;

    let command = Command::parse();

    let mut pipeline = match command {
        Command::RaydiumAccountsRpc => make_raydium_rpc_accounts_pipeline()?,
        Command::RaydiumInstructionsRpc => {
            make_raydium_rpc_instruction_pipeline(kv_store, message_queue, db)?
        }
    };

    let price_cache = SOL_PRICE_CACHE.clone();

    tokio::spawn(async move {
        if let Err(e) = price_cache.start_price_stream().await {
            error!("Error in SOL price stream: {}", e);
        }
    });

    pipeline.run().await?;

    Ok(())
}

#[cfg(not(feature = "rpc"))]
fn main() -> Result<()> {
    println!("rpc is not enabled, cargo run --bin rpc-crawler --features rpc");
    Ok(())
}
