use flexi_logger::{colored_detailed_format, Duplicate, Logger, WriteMode};
use jito_protos::searcher::{MempoolSubscription, NextScheduledLeaderRequest};
use jito_searcher_client::get_searcher_client;
use raydium_library::amm;
use std::{
    error::Error,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use util::env;

use clap::Parser;
use listen::{
    address,
    app::{App, Command},
    buyer, buyer_service, checker, checker_service, constants,
    jup::Jupiter,
    listener_service, prometheus,
    raydium::{self, Raydium, SwapArgs},
    rpc, seller, seller_service, tx_parser, util, BlockAndProgramSubscribable, Listener, Provider,
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

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::from_filename(".env").unwrap();

    let _logger = Logger::try_with_str("info")?
        .format(colored_detailed_format)
        .write_mode(WriteMode::Async)
        .duplicate_to_stdout(Duplicate::Info)
        .start()?;

    let app = App::parse();

    if app.args.tokio_console.unwrap_or(false) {
        console_subscriber::init();
    }

    // 30th April, let's see how well this ages lol (was 135.)
    // 13th May, still going strong with the algo, now at 145
    // 16th May 163, I paperhanded 20+ SOL :(
    // 28th May - SOL was for 190ish, dipped and longing now
    let sol_price = 163.;

    match app.command {
        Command::GenerateCustomAddress { prefixes } => {
            let found_flag = Arc::new(AtomicBool::new(false));
            // note that this will only spawn as many workers as the runtime allows
            // defaults to num vcpus
            let workers: Vec<_> = (0..8)
                .map(|_| {
                    let prefixes = prefixes.clone();
                    let found_flag = Arc::clone(&found_flag);
                    tokio::spawn(async move {
                        address::generate_custom_sol_address(prefixes, found_flag).await;
                    })
                })
                .collect();
            for worker in workers {
                worker.await?;
            }
        }
        Command::Ata { mint } => {
            let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read wallet");
            info!(
                "ATA: {:?}",
                spl_associated_token_account::get_associated_token_address(
                    &wallet.pubkey(),
                    &Pubkey::from_str(&mint)?
                )
            );
        }
        Command::SplStream { ata } => {
            let ata = Pubkey::from_str(&ata)?;
            let pubsub_client =
                nonblocking::pubsub_client::PubsubClient::new(env("WS_URL").as_str()).await?;
            seller::get_spl_balance_stream(&pubsub_client, &ata).await?;
        }
        Command::Checks { signature } => {
            let (ok, checklist) = checker::run_checks(signature).await?;
            println!("ok? {}, {:?}", ok, checklist);
        }
        Command::Blockhash {} => {
            let provider = Provider::new(env("RPC_URL").to_string());
            for _ in 0..3 {
                let start = std::time::Instant::now();
                let res = provider.rpc_client.get_latest_blockhash().await?;
                println!("{:?}", res);
                println!("Time elapsed: {:?}", start.elapsed());
            }
        }
        Command::Snipe {} => {
            let results = tokio::join!(
                buyer_service::run_buyer_service(),
                checker_service::run_checker_service(),
                listener_service::run_listener_webhook_service()
            );
            results.0?;
            results.1?;
            results.2?;
        }
        Command::ParsePool { signature } => {
            let provider = Provider::new(env("RPC_URL").to_string());
            let new_pool = tx_parser::parse_new_pool(&provider.get_tx(signature.as_str()).await?)?;
            println!("{:?}", new_pool);
        }
        Command::TopHolders { mint } => {
            let provider = Provider::new(env("RPC_URL").to_string());
            let mint = Pubkey::from_str(mint.as_str()).unwrap();
            let (_, ok) = buyer::check_top_holders(&mint, &provider).await?;
            info!("Top holders check passed: {}", ok);
        }
        Command::ListenForSolPooled { amm_pool } => {
            let provider = Provider::new(env("RPC_URL").to_string());
            let pubsub_client =
                nonblocking::pubsub_client::PubsubClient::new(env("WS_URL").as_str()).await?;
            buyer::listen_for_sol_pooled(
                &Pubkey::from_str(amm_pool.as_str())?,
                &provider.rpc_client,
                &pubsub_client,
            )
            .await?;
        }
        Command::ListenForBurn { amm_pool } => {
            let provider = Provider::new(env("RPC_URL").to_string());
            let pubsub_client =
                nonblocking::pubsub_client::PubsubClient::new(env("WS_URL").as_str()).await?;
            buyer::listen_for_burn(
                &Pubkey::from_str(amm_pool.as_str())?,
                &provider.rpc_client,
                &pubsub_client,
            )
            .await?;
        }
        Command::TrackPosition { amm_pool, owner } => {
            let provider = Provider::new(env("RPC_URL").to_string());
            let amm_pool = Pubkey::from_str(amm_pool.as_str()).expect("amm pool is a valid pubkey");

            let amm_program = Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)?;
            // load amm keys
            let amm_keys =
                amm::utils::load_amm_keys(&provider.rpc_client, &amm_program, &amm_pool).await?;
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
                && market_keys.pc_mint.to_string() != constants::SOLANA_PROGRAM_ID
            {
                error!("pool is not against solana");
                return Ok(());
            }
            let coin_mint_is_sol =
                market_keys.coin_mint.to_string() == constants::SOLANA_PROGRAM_ID;
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

                raydium::calc_result_to_financials(coin_mint_is_sol, result, owner_balance);

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        Command::MonitorMempool {} => {
            let auth =
                Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).expect("read auth keypair");
            let mut searcher_client =
                get_searcher_client(&env("BLOCK_ENGINE_URL"), &Arc::new(auth))
                    .await
                    .expect("makes searcher client");
            let res = searcher_client
                .subscribe_mempool(MempoolSubscription {
                    ..Default::default()
                })
                .await?;
            info!("{:?}", res);
        }
        Command::MonitorLeaders {} => {
            let regions = vec![
                "frankfurt".to_string(),
                "amsterdam".to_string(),
                "tokyo".to_string(),
                "ny".to_string(),
            ];
            let auth = Arc::new(Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap());
            let mut searcher_client = get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                .await
                .expect("makes searcher client");
            for region in regions {
                let res = searcher_client
                    .get_next_scheduled_leader(NextScheduledLeaderRequest {
                        regions: vec![region],
                    })
                    .await
                    .unwrap();
                info!("{:?}", res.into_inner());
            }

            drop(searcher_client);
        }
        Command::MonitorSlots {} => {
            Listener::new(env("WS_URL").to_string()).slot_subscribe()?;
            let auth = Arc::new(Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap());
            let mut searcher_client = get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                .await
                .expect("makes searcher client");
            let _ = searcher_client
                .subscribe_mempool(MempoolSubscription {
                    ..Default::default()
                })
                .await;
        }
        Command::BenchRPC { rpc_url } => rpc::eval_rpc(rpc_url.as_str()),
        Command::PriorityFee {} => {
            let provider = Provider::new(env("RPC_URL").to_string());
            println!(
                "{:?}",
                provider
                    .rpc_client
                    .get_recent_prioritization_fees(
                        vec![
                            Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY).unwrap()
                        ]
                        .as_slice()
                    )
                    .await
            );
        }
        Command::Price { amm_pool } => {
            let provider = Provider::new(env("RPC_URL").to_string());
            let pubsub_client =
                nonblocking::pubsub_client::PubsubClient::new(env("WS_URL").as_str()).await?;
            let amm_pool = Pubkey::from_str(amm_pool.as_str())?;
            seller::listen_price(&amm_pool, &provider.rpc_client, &pubsub_client)
                .await
                .expect("listen price");
        }
        Command::CheckerService {} => {
            checker_service::run_checker_service().await?;
        }
        Command::BuyerService {} => {
            buyer_service::run_buyer_service().await?;
        }
        Command::SellerService {} => {
            seller_service::run_seller_service().await?;
        }
        Command::ListenerService { webhook } => {
            let webhook = webhook.unwrap_or(false);
            if webhook {
                listener_service::run_listener_webhook_service().await?;
            } else {
                listener_service::run_listener_pubsub_service().await?;
            }
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
            let provider = Provider::new(env("RPC_URL").to_string());
            let raydium = Raydium::new();
            let jup = Jupiter::new();
            let start = std::time::Instant::now();
            if input_mint == "sol" {
                input_mint = constants::SOLANA_PROGRAM_ID.to_string();
            }
            if output_mint == "sol" {
                output_mint = constants::SOLANA_PROGRAM_ID.to_string();
            }
            let path = match app.args.keypair_path {
                Some(path) => path,
                None => std::env::var("HOME").expect("HOME is set") + "/.config/solana/id.json",
            };
            if dex.unwrap_or("".to_string()) == "raydium" {
                // TODO check out solend also
                let amm_pool_id = Pubkey::from_str(amm_pool_id.unwrap().as_str())?;
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
                    .swap(SwapArgs {
                        amm_pool: amm_pool_id,
                        input_token_mint,
                        output_token_mint,
                        amount: amount_specified,
                        slippage: slippage_bps,
                        wallet,
                        provider,
                        confirmed: yes.unwrap_or(false),
                    })
                    .await?;
                return Ok(());
            }
            let keypair = Keypair::read_from_file(&path)?;
            if let Some(amount) = amount {
                jup.swap(SwapArgs {
                    amm_pool: Pubkey::default(),
                    input_token_mint: Pubkey::from_str(&input_mint)?,
                    output_token_mint: Pubkey::from_str(&output_mint)?,
                    amount: amount as u64,
                    wallet: keypair,
                    provider,
                    confirmed: yes.unwrap_or(false),
                    slippage: slippage.unwrap_or(800) as u64,
                })
                .await?;
            } else {
                jup.swap_entire_balance(
                    input_mint,
                    output_mint,
                    keypair,
                    provider,
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
            let provider = Provider::new(env("RPC_URL"));
            let path = match app.args.keypair_path {
                Some(path) => path,
                None => std::env::var("HOME").expect("HOME is set") + "/.config/solana/id.json",
            };
            let keypair = Keypair::read_from_file(&path)?;

            info!("Pubkey: {}", keypair.pubkey());
            let balance = provider.get_balance(&keypair.pubkey()).await?;
            info!("Balance: {} lamports", balance);
        }
        Command::Tx { signature } => {
            let provider = Provider::new(env("RPC_URL").to_string());
            let tx = provider.get_tx(signature.as_str()).await?;
            info!("Tx: {}", serde_json::to_string_pretty(&tx)?);
            let mint = tx_parser::parse_mint(&tx)?;
            let pricing = provider.get_pricing(&mint).await?;
            info!("Pricing: {:?}", pricing);

            let swap = tx_parser::parse_swap(&tx)?;
            info!("Swap: {}", serde_json::to_string_pretty(&swap)?);

            let sol_notional = listen::util::lamports_to_sol(swap.quote_amount as u64);
            let usd_notional = sol_notional * sol_price;
            info!("{} ({} USD)", sol_notional, usd_notional);

            return Ok(());
        }
        Command::Listen {
            worker_count,
            buffer_size,
        } => {
            let listener = Listener::new(env("WS_URL"));
            let (transactions_received, transactions_processed, registry) =
                prometheus::setup_metrics();

            // Start the metrics server
            let metrics_server = tokio::spawn(async move {
                prometheus::run_metrics_server(registry).await;
            });

            let (mut subs, recv) = listener.logs_subscribe()?; // Subscribe to logs

            let (tx, rx) =
                tokio::sync::mpsc::channel::<Response<RpcLogsResponse>>(buffer_size as usize);
            let rx = Arc::new(Mutex::new(rx));

            // Worker tasks, increase in prod to way more, talking min 30-50
            let pool: Vec<_> = (0..worker_count as usize)
                .map(|_| {
                    let rx = Arc::clone(&rx);
                    let provider = Provider::new(env("RPC_URL").to_string());
                    let transactions_processed = transactions_processed.clone();
                    tokio::spawn(async move {
                        while let Some(log) = rx.lock().await.recv().await {
                            let tx = {
                                match provider.get_tx(&log.value.signature).await {
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
                            let lamports = tx_parser::parse_notional(&tx).ok().unwrap();
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
                while let Ok(log) = recv.recv_timeout(Duration::from_secs(10)) {
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
