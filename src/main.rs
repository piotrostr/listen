use dotenv;
use jito_searcher_client::get_searcher_client;
use log::{warn, LevelFilter};
use std::{str::FromStr, sync::Arc, time::Duration};

use clap::Parser;
use listen::{
    constants, jito,
    jup::Jupiter,
    prometheus,
    raydium::Raydium,
    rpc, tx_parser,
    util::{self, must_get_env},
    Listener, Provider,
};
use solana_client::{
    nonblocking,
    rpc_response::{Response, RpcLogsResponse},
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
};
use tokio::sync::Mutex;

use log::{error, info};

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
    BenchRPC {
        #[arg(long)]
        rpc_url: String,
    },
    PriorityFee {},
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
    ListenPools {
        #[arg(long, action = clap::ArgAction::SetTrue)]
        snipe: Option<bool>,

        #[arg(long, action = clap::ArgAction::SetTrue)]
        use_jito: Option<bool>,
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
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();

    // 30th April, let's see how well this ages lol
    let sol_price = 135.;
    let app = App::parse();
    let provider = Provider::new(must_get_env("RPC_URL"));
    let raydium = Raydium::new();
    let listener = Listener::new(app.args.ws_url);
    let jup = Jupiter::new();

    let auth = Arc::new(
        Keypair::read_from_file(dotenv::var("AUTH_KEYPAIR_PATH").unwrap())
            .unwrap(),
    );
    let mut searcher_client = get_searcher_client(
        dotenv::var("BLOCK_ENGINE_URL").unwrap().as_str(),
        &auth,
    )
    .await
    .unwrap();
    match app.command {
        Command::BenchRPC { rpc_url } => rpc::eval_rpc(rpc_url.as_str()),
        Command::PriorityFee {} => {
            println!(
                "{:?}",
                provider.rpc_client.get_recent_prioritization_fees(
                    vec![Pubkey::from_str(
                        constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY
                    )
                    .unwrap()]
                    .as_slice()
                )
            );
        }
        Command::ListenPools { snipe, use_jito } => {
            let snipe = snipe.unwrap_or(false);
            let use_jito = use_jito.unwrap_or(false);
            let (mut subs, recv) = listener.new_lp_subscribe()?;
            // let provider = Provider::new(app.args.url);
            println!("Listening for LP events");
            let wallet = Keypair::read_from_file(
                must_get_env("HOME") + "/.config/solana/id.json",
            )?;
            let listener = tokio::spawn(async move {
                while let Ok(log) = recv.recv() {
                    if log.value.err.is_some() {
                        continue; // Skip error logs
                    }
                    println!("{}", serde_json::to_string_pretty(&log).unwrap());
                    let new_pool_info = tx_parser::parse_new_pool(
                        &provider.get_tx(&log.value.signature).unwrap(),
                    )
                    .unwrap();
                    println!("{:?}", new_pool_info);
                    // TODO move this to a separate service listening in a separate thread
                    // same as in case of receiver and processor pool for Command::Listen
                    if snipe {
                        if use_jito {
                            let ixs = raydium
                                .make_swap_ixs(
                                    new_pool_info.amm_pool_id,
                                    new_pool_info.input_mint,
                                    new_pool_info.output_mint,
                                    300,
                                    10_000_000,
                                    true,
                                    &provider,
                                    &wallet,
                                )
                                .expect("makes swap ixs");
                            match jito::send_swap_tx(
                                ixs,
                                50000,
                                &wallet,
                                &mut searcher_client,
                                &nonblocking::rpc_client::RpcClient::new(
                                    must_get_env("RPC_URL"),
                                ),
                            )
                            .await
                            {
                                Ok(_) => info!("Bundle OK"),
                                Err(e) => {
                                    warn!("swap tx: {}", e)
                                }
                            }
                        } else {
                            raydium
                                .swap(
                                    new_pool_info.amm_pool_id,
                                    new_pool_info.input_mint,
                                    new_pool_info.output_mint,
                                    300,    // 3.0%
                                    100000, // 0.001 SOL
                                    true,
                                    &wallet,
                                    &provider,
                                    false,
                                )
                                .unwrap();
                        }
                    }
                }
                subs.shutdown().unwrap(); // Shutdown subscription on exit
            });
            listener.await?;
            return Ok(());
        }
        Command::Swap {
            mut input_mint,
            mut output_mint,
            amount,
            slippage,
            yes,
            dex,
            amm_pool_id,
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
            if dex.unwrap_or("".to_string()) == "raydium" {
                // TODO check out solend also
                let amm_pool_id =
                    Pubkey::from_str(amm_pool_id.unwrap().as_str())?;
                let input_token_mint = Pubkey::from_str(input_mint.as_str())?;
                let output_token_mint = Pubkey::from_str(output_mint.as_str())?;
                let slippage_bps = slippage.unwrap_or(800) as u64; // 8%
                let wallet = Keypair::read_from_file(path)?;
                info!("Wallet: {}", wallet.pubkey());
                info!(
                    "Balance (lamports): {}",
                    provider.get_balance(&wallet.pubkey())?
                );
                let amount_specified = if amount.is_some() {
                    amount.unwrap() as u64
                } else {
                    provider
                        .get_spl_balance(&wallet.pubkey(), &input_token_mint)?
                };
                let swap_base_in = true;
                raydium.swap(
                    amm_pool_id,
                    input_token_mint,
                    output_token_mint,
                    slippage_bps,
                    amount_specified,
                    swap_base_in,
                    &wallet,
                    &provider,
                    yes.unwrap_or(false),
                )?;
                return Ok(());
            }
            let keypair = Keypair::read_from_file(&path)?;
            if let Some(amount) = amount {
                jup.swap(
                    input_mint,
                    output_mint,
                    amount as u64,
                    &keypair,
                    &provider,
                    yes.unwrap_or(false),
                    slippage.unwrap_or(800),
                )
                .await?;
            } else {
                jup.swap_entire_balance(
                    input_mint,
                    output_mint,
                    &keypair,
                    &provider,
                    yes.unwrap_or(false),
                    slippage.unwrap_or(50),
                )
                .await?;
            }
            let duration = start.elapsed();
            info!("Time elapsed: {:?}", duration);
            return Ok(());
        }
        Command::Wallet {} => {
            let path = match app.args.keypair_path {
                Some(path) => path,
                None => util::must_get_env("HOME") + "/.config/solana/id.json",
            };
            let keypair = Keypair::read_from_file(&path)?;

            info!("Pubkey: {}", keypair.pubkey());
            let balance = provider.get_balance(&keypair.pubkey())?;
            info!("Balance: {} lamports", balance);
        }
        Command::Tx { signature } => {
            let tx = provider.get_tx(signature.as_str())?;
            info!("Tx: {}", serde_json::to_string_pretty(&tx)?);
            let mint = tx_parser::parse_mint(&tx)?;
            let pricing = provider.get_pricing(&mint).await?;
            info!("Pricing: {:?}", pricing);

            let swap = tx_parser::parse_swap(&tx)?;
            info!("Swap: {}", serde_json::to_string_pretty(&swap)?);

            let sol_notional =
                listen::util::lamports_to_sol(swap.quote_amount as u64);

            let usd_notional = sol_notional * sol_price;

            info!("{} ({} USD)", sol_notional, usd_notional);

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
                                        info!(
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
                            info!(
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
                        Err(e) => error!("Failed to send log: {}", e),
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
