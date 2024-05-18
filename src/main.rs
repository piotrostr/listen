use dotenv_codegen::dotenv;
use flexi_logger::{Duplicate, FileSpec, Logger, WriteMode};
use jito_protos::searcher::{MempoolSubscription, NextScheduledLeaderRequest};
use jito_searcher_client::get_searcher_client;
use raydium_library::amm;
use std::{error::Error, str::FromStr, sync::Arc, time::Duration};

use clap::Parser;
use listen::{
    buyer, buyer_service, constants,
    jup::Jupiter,
    prometheus,
    raydium::{self, Raydium},
    rpc, snipe, tx_parser, util, BlockAndProgramSubscribable, Listener,
    Provider,
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
    ListenForSolPooled {
        #[arg(long)]
        amm_pool: String,
    },
    BuyerService {},
    TrackPosition {
        #[arg(long)]
        amm_pool: String,

        #[arg(long)]
        owner: String,
    },
    TopHolders {
        #[arg(long)]
        mint: String,
    },
    MonitorLeaders {},
    MonitorSlots {},
    Price {
        #[arg(long)]
        mint: String,
    },
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
    ListenForBurn {
        #[arg(long)]
        amm_pool: String,
    },
    ListenerService {},
    Snipe {},
    Wallet {},
    ParsePool {
        #[arg(long)]
        signature: String,
    },
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

fn log_format(
    writer: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> std::io::Result<()> {
    write!(
        writer,
        "{} [{}] {} [{}:{}] {}\n",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        record.target(),
        record.file().unwrap_or("<unknown>"),
        record.line().unwrap_or(0),
        record.args()
    )
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn Error>> {
    let _logger = Logger::try_with_str("info")?
        .log_to_file(FileSpec::default())
        .format(log_format)
        .write_mode(WriteMode::Async)
        .duplicate_to_stdout(Duplicate::Info)
        .start()?;

    // 30th April, let's see how well this ages lol (was 135.)
    // 13th May, still going strong with the algo, now at 145
    // 16th May 163, I paperhanded 20+ SOL :(
    let sol_price = 163.;
    let app = App::parse();
    let provider = Provider::new(dotenv!("RPC_URL").to_string());
    let raydium = Raydium::new();
    let listener = Listener::new(dotenv!("WS_URL").to_string());
    let jup = Jupiter::new();

    let auth = Arc::new(
        Keypair::read_from_file(dotenv!("AUTH_KEYPAIR_PATH")).unwrap(),
    );
    let mut searcher_client =
        get_searcher_client(dotenv!("BLOCK_ENGINE_URL"), &auth)
            .await
            .expect("makes searcher client");
    match app.command {
        Command::Snipe {} => {
            let (listener_res, buyer_res) = tokio::join!(
                snipe::run_listener(),
                buyer_service::run_buyer_service()
            );
            listener_res?;
            buyer_res?;
        }
        Command::ParsePool { signature } => {
            let new_pool = tx_parser::parse_new_pool(
                &provider.get_tx(signature.as_str()).await?,
            )?;
            println!("{:?}", new_pool);
        }
        Command::BuyerService {} => {
            buyer_service::run_buyer_service().await?;
        }
        Command::TopHolders { mint } => {
            let mint = Pubkey::from_str(mint.as_str()).unwrap();
            let ok = buyer::check_top_holders(&mint, &provider).await?;
            info!("Top holders check passed: {}", ok);
        }
        Command::ListenForSolPooled { amm_pool } => {
            let pubsub_client = nonblocking::pubsub_client::PubsubClient::new(
                dotenv!("WS_URL"),
            )
            .await?;
            buyer::listen_for_sol_pooled(
                &Pubkey::from_str(amm_pool.as_str())?,
                &provider.rpc_client,
                &pubsub_client,
            )
            .await?;
        }
        Command::ListenForBurn { amm_pool } => {
            let pubsub_client = nonblocking::pubsub_client::PubsubClient::new(
                dotenv!("WS_URL"),
            )
            .await?;
            buyer::listen_for_burn(
                &Pubkey::from_str(amm_pool.as_str())?,
                &provider.rpc_client,
                &pubsub_client,
            )
            .await?;
        }
        Command::TrackPosition { amm_pool, owner } => {
            let amm_pool = Pubkey::from_str(amm_pool.as_str())
                .expect("amm pool is a valid pubkey");

            let amm_program =
                Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)?;
            // load amm keys
            let amm_keys = amm::utils::load_amm_keys(
                &provider.rpc_client,
                &amm_program,
                &amm_pool,
            )
            .await?;
            info!("{:?}", amm_keys);
            // load market keys
            let market_keys = amm::openbook::get_keys_for_market(
                &provider.rpc_client,
                &amm_keys.market_program,
                &amm_keys.market,
            )
            .await?;
            info!("{:?}", market_keys);
            if market_keys.coin_mint.to_string() != constants::SOLANA_PROGRAM_ID
                && market_keys.pc_mint.to_string()
                    != constants::SOLANA_PROGRAM_ID
            {
                error!("pool is not against solana");
                return Ok(());
            }
            let coin_mint_is_sol = market_keys.coin_mint.to_string()
                == constants::SOLANA_PROGRAM_ID;
            let owner_balance = provider
                .get_spl_balance(
                    &Pubkey::from_str(owner.as_str()).unwrap(),
                    if coin_mint_is_sol {
                        &market_keys.pc_mint
                    } else {
                        &market_keys.coin_mint
                    },
                )
                .await?;

            loop {
                let result = amm::calculate_pool_vault_amounts(
                    &provider.rpc_client,
                    &amm_program,
                    &amm_pool,
                    &amm_keys,
                    &market_keys,
                    amm::CalculateMethod::CalculateWithLoadAccount,
                )
                .await?;

                raydium::calc_result_to_financials(
                    coin_mint_is_sol,
                    result,
                    owner_balance,
                );

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        Command::MonitorLeaders {} => {
            let regions = vec![
                "frankfurt".to_string(),
                "amsterdam".to_string(),
                "tokyo".to_string(),
                "ny".to_string(),
            ];
            for region in regions {
                let res = searcher_client
                    .get_next_scheduled_leader(NextScheduledLeaderRequest {
                        regions: vec![region],
                    })
                    .await
                    .unwrap();
                info!("{:?}", res.into_inner());
            }
        }
        Command::MonitorSlots {} => {
            listener.slot_subscribe()?;
            let _ = searcher_client
                .subscribe_mempool(MempoolSubscription {
                    ..Default::default()
                })
                .await;
        }
        Command::BenchRPC { rpc_url } => rpc::eval_rpc(rpc_url.as_str()),
        Command::PriorityFee {} => {
            println!(
                "{:?}",
                provider
                    .rpc_client
                    .get_recent_prioritization_fees(
                        vec![Pubkey::from_str(
                            constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY
                        )
                        .unwrap()]
                        .as_slice()
                    )
                    .await
            );
        }
        Command::Price { mint } => {
            println!("{}", mint);
            // not implemented
        }
        Command::ListenerService {} => {
            snipe::run_listener().await?;
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
                None => {
                    std::env::var("HOME").expect("HOME is set")
                        + "/.config/solana/id.json"
                }
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
                    provider.get_balance(&wallet.pubkey()).await?
                );
                let amount_specified = if amount.is_some() {
                    amount.unwrap() as u64
                } else {
                    provider
                        .get_spl_balance(&wallet.pubkey(), &input_token_mint)
                        .await?
                };
                raydium
                    .swap(
                        amm_pool_id,
                        input_token_mint,
                        output_token_mint,
                        amount_specified,
                        slippage_bps,
                        &wallet,
                        &provider,
                        yes.unwrap_or(false),
                    )
                    .await?;
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
                None => {
                    std::env::var("HOME").expect("HOME is set")
                        + "/.config/solana/id.json"
                }
            };
            let keypair = Keypair::read_from_file(&path)?;

            info!("Pubkey: {}", keypair.pubkey());
            let balance = provider.get_balance(&keypair.pubkey()).await?;
            info!("Balance: {} lamports", balance);
        }
        Command::Tx { signature } => {
            let tx = provider.get_tx(signature.as_str()).await?;
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
                    let provider =
                        Provider::new(dotenv!("RPC_URL").to_string());
                    let transactions_processed = transactions_processed.clone();
                    tokio::spawn(async move {
                        while let Some(log) = rx.lock().await.recv().await {
                            let tx = {
                                match provider
                                    .get_tx(&log.value.signature)
                                    .await
                                {
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
