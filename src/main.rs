use std::{str::FromStr, sync::Arc, time::Duration};

use clap::Parser;
use listen::{constants, prometheus, tx_parser, util, Listener, Provider};
use solana_client::rpc_response::{Response, RpcLogsResponse};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
pub struct App {
    #[clap(flatten)]
    args: Args,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    url: String,

    #[arg(short, long, default_value = "wss://api.mainnet-beta.solana.com")]
    ws_url: String,

    #[arg(short, long)]
    keypair_path: Option<String>,
}

#[derive(Debug, Parser)]
enum Command {
    Tx {
        #[arg(short, long)]
        signature: String,
    },
    Listen {
        #[arg(long, default_value_t = 10)]
        worker_count: i32,

        #[arg(long, default_value_t = 10)]
        buffer_size: i32,
    },

    Wallet {},

    Swap {
        #[arg(long)]
        input_mint: String,
        #[arg(long)]
        output_mint: String,
        #[arg(long)]
        amount: Option<i64>,
        #[arg(long)]
        slippage: Option<u16>,
        #[arg(long)]
        dex: Option<String>,
        #[arg(long)]
        amm_pool_id: Option<String>,

        #[clap(short, long, action = clap::ArgAction::SetTrue)]
        yes: Option<bool>,

        #[clap(long, action = clap::ArgAction::SetTrue)]
        testnet: Option<bool>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 30th April, let's see how well this ages lol
    let sol_price = 135.;
    let app = App::parse();
    match app.command {
        Command::Swap {
            mut input_mint,
            mut output_mint,
            amount,
            slippage,
            yes,
            dex,
            amm_pool_id,
            testnet,
        } => {
            let start = std::time::Instant::now();
            if input_mint == "sol" {
                input_mint = constants::SOLANA_PROGRAM_ID.to_string();
            }
            if output_mint == "sol" {
                output_mint = constants::SOLANA_PROGRAM_ID.to_string();
            }
            let path = match app.args.keypair_path {
                Some(path) => path,
                None => util::must_get_env("HOME") + "/.config/solana/id.json",
            };
            let raydium = listen::raydium::Raydium::new();
            if dex.unwrap_or("".to_string()) == "raydium" {
                let testnet = testnet.unwrap_or(false);
                let amm_program = Pubkey::from_str(if testnet {
                    constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY_TESTNET
                } else {
                    constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY
                })?;
                let rpc_url = if testnet {
                    "https://api.devnet.solana.com".to_string()
                } else {
                    util::must_get_env("RPC_URL")
                    // "https://api.mainnet-beta.solana.com".to_string()
                };
                // TODO check out solend also
                let provider = Provider::new(rpc_url);
                let amm_pool_id =
                    Pubkey::from_str(amm_pool_id.unwrap().as_str())?;
                let input_token_mint = Pubkey::from_str(input_mint.as_str())?;
                let output_token_mint = Pubkey::from_str(output_mint.as_str())?;
                let slippage_bps = slippage.unwrap_or(50) as u64;
                let wallet = Keypair::read_from_file(path)?;
                println!("Wallet: {}", wallet.pubkey());
                println!(
                    "Balance: {}",
                    provider.get_balance(&wallet.pubkey())?
                );
                let amount_specified = if amount.is_some() {
                    amount.unwrap() as u64
                } else {
                    let spl_token_balance = provider
                        .get_spl_balance(&wallet.pubkey(), &input_token_mint)?;
                    spl_token_balance
                };
                let swap_base_in = true;
                raydium.swap(
                    amm_program,
                    amm_pool_id,
                    input_token_mint,
                    output_token_mint,
                    slippage_bps,
                    amount_specified,
                    swap_base_in,
                    &wallet,
                    &provider,
                )?;
                return Ok(());
            }
            let provider = Provider::new(app.args.url);
            let jup = listen::jup::Jupiter::new(slippage.unwrap_or(50));
            let keypair = Keypair::read_from_file(&path)?;
            if let Some(amount) = amount {
                jup.swap(
                    input_mint,
                    output_mint,
                    amount as u64,
                    &keypair,
                    &provider,
                    yes.unwrap_or(false),
                )
                .await?;
            } else {
                jup.swap_entire_balance(
                    input_mint,
                    output_mint,
                    &keypair,
                    &provider,
                    yes.unwrap_or(false),
                )
                .await?;
            }
            let duration = start.elapsed();
            println!("Time elapsed: {:?}", duration);
            return Ok(());
        }
        Command::Wallet {} => {
            let path = match app.args.keypair_path {
                Some(path) => path,
                None => util::must_get_env("HOME") + "/.config/solana/id.json",
            };
            let keypair = Keypair::read_from_file(&path)?;
            println!("path: {}", path);
            let provider = Provider::new(app.args.url);

            println!("Pubkey: {}", keypair.pubkey());
            let balance = provider.get_balance(&keypair.pubkey())?;
            println!("Balance: {} lamports", balance);
        }
        Command::Tx { signature } => {
            let provider = Provider::new(app.args.url);
            let tx = provider.get_tx(signature.as_str())?;
            println!("Tx: {}", serde_json::to_string_pretty(&tx)?);
            let mint = tx_parser::parse_mint(&tx)?;
            let pricing = provider.get_pricing(&mint).await?;
            println!("Pricing: {:?}", pricing);

            let swap = tx_parser::parse_swap(&tx)?;
            println!("Swap: {}", serde_json::to_string_pretty(&swap)?);

            let sol_notional =
                listen::util::lamports_to_sol(swap.quote_amount as u64);

            let usd_notional = sol_notional * sol_price;

            println!("{} ({} USD)", sol_notional, usd_notional);

            return Ok(());
        }
        Command::Listen {
            worker_count,
            buffer_size,
        } => {
            let (transactions_received, transactions_processed, registry) =
                prometheus::setup_metrics();

            // Start the metrics server
            let metrics_server = tokio::spawn(async move {
                prometheus::run_metrics_server(registry).await;
            });

            let listener = Listener::new(app.args.ws_url);
            let (mut subs, recv) = listener.logs_subscribe()?; // Subscribe to logs

            let (tx, rx) = tokio::sync::mpsc::channel::<
                Response<RpcLogsResponse>,
            >(buffer_size as usize);
            let rx = Arc::new(Mutex::new(rx));

            // Worker tasks, increase in prod to way more, talking min 30-50
            let pool: Vec<_> = (0..worker_count as usize)
                .map(|_| {
                    let rx = Arc::clone(&rx);
                    let provider = Provider::new(util::must_get_env("RPC_URL"));
                    let transactions_processed = transactions_processed.clone();
                    tokio::spawn(async move {
                        while let Some(log) = rx.lock().await.recv().await {
                            let tx = {
                                match provider.get_tx(&log.value.signature) {
                                    Ok(tx) => tx,
                                    Err(e) => {
                                        println!(
                                            "Failed to get tx: {}; sig: {}",
                                            e, log.value.signature
                                        );
                                        continue;
                                    }
                                }
                            };
                            let lamports =
                                tx_parser::parse_notional(&tx).ok().unwrap();
                            let sol_notional = util::lamports_to_sol(lamports);
                            transactions_processed.inc();
                            if sol_notional < 10. {
                                continue;
                            }
                            println!(
                                "https://solana.fm/tx/{}: {} SOL",
                                &log.value.signature, sol_notional,
                            );
                        }
                    })
                })
                .collect();

            // Log receiving task
            let log_receiver = tokio::spawn(async move {
                let transactions_received = transactions_received.clone();
                while let Ok(log) = recv.recv_timeout(Duration::from_secs(1)) {
                    if log.value.err.is_some() {
                        continue; // Skip error logs
                    }
                    match tx.send(log.clone()).await {
                        Err(e) => println!("Failed to send log: {}", e),
                        Ok(_) => {
                            transactions_received.inc();
                        }
                    }
                }
                drop(tx);
                subs.shutdown().unwrap(); // Shutdown subscription on exit
            });

            // Await all tasks
            log_receiver.await?;
            metrics_server.await?;
            for worker in pool {
                worker.await?;
            }
        }
    }

    Ok(())
}
