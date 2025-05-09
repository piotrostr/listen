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
    address, agent,
    app::{App, Command},
    ata, buyer, buyer_service, checker, checker_service, constants,
    jup::Jupiter,
    listener_service, prometheus,
    pump::{self},
    pump_service,
    raydium::{self, Raydium, SwapArgs},
    rpc, seller, seller_service,
    service::run_listen_service,
    tx_parser, util, BlockAndProgramSubscribable, Listener, Provider,
};
use solana_client::{
    nonblocking::{self, rpc_client::RpcClient},
    rpc_response::{Response, RpcLogsResponse},
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
};
use tokio::sync::Mutex;

use log::{error, info, warn};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let _logger =
        Logger::try_with_str(std::option_env!("RUST_LOG").unwrap_or("info"))?
            .format(colored_detailed_format)
            .write_mode(WriteMode::Async)
            .duplicate_to_stdout(Duplicate::Info)
            .start()?;

    let app = App::parse();

    if app.args.tokio_console.unwrap_or(false) {
        console_subscriber::init();
    }

    let sol_price = 210.;

    match app.command {
        Command::ListenService { port: _port } => {
            run_listen_service().await?;
        }
        Command::ArcAgent {} => {
            agent::make_agent().await.expect("make agent");
        }
        Command::BundleStatus { bundle } => {
            let client = reqwest::Client::new();
            let url = "https://mainnet.block-engine.jito.wtf/api/v1/bundles";

            let payload = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getBundleStatuses",
                "params": [[bundle]]
            });

            let response = client
                .post(url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await?;

            let result: serde_json::Value = response.json().await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::DownloadRaydiumJson { update } => {
            raydium::download_raydium_json(update.unwrap_or(false)).await?;
        }
        // sweep raydium burns all of the tokens,
        // dont use it unless you know what you're doing
        Command::SweepRaydium { wallet_path } => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            raydium::sweep_raydium(&rpc_client, wallet_path).await?;
            // return Err("Unimplemented (danger zone)".into());
        }
        Command::CloseTokenAccounts { wallet_path } => {
            let keypair =
                Keypair::read_from_file(wallet_path).expect("read wallet");
            info!("Wallet: {}", keypair.pubkey());
            let rpc_client = Arc::new(RpcClient::new(env("RPC_URL")));
            ata::close_all_atas(rpc_client, &keypair).await?;
        }
        Command::PumpService {} => {
            pump_service::run_pump_service().await?;
        }
        Command::GrabMetadata { mint } => {
            pump::fetch_metadata(&Pubkey::from_str(&mint)?).await?;
        }
        Command::SellPump { mint } => {
            let keypair = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .expect("read wallet");
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let ata =
                spl_associated_token_account::get_associated_token_address(
                    &keypair.pubkey(),
                    &Pubkey::from_str(&mint)?,
                );

            let actual_balance = rpc_client
                .get_token_account_balance(&ata)
                .await?
                .amount
                .parse::<u64>()?;

            let pump_accounts =
                pump::mint_to_pump_accounts(&Pubkey::from_str(&mint)?).await?;

            pump::sell_pump_token(
                &keypair,
                &rpc_client,
                pump_accounts,
                actual_balance,
            )
            .await?;
        }
        Command::BumpPump { mint } => {
            let keypair = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .expect("read wallet");
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let auth = Arc::new(
                Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap(),
            );
            let mut searcher_client = Arc::new(Mutex::new(
                get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                    .await
                    .expect("makes searcher client"),
            ));
            loop {
                match pump::send_pump_bump(
                    &keypair,
                    &rpc_client,
                    &Pubkey::from_str(&mint)?,
                    &mut searcher_client,
                    true,
                )
                .await
                {
                    Ok(_) => {
                        info!("Bump success");
                    }
                    Err(e) => {
                        warn!("Bump failed: {}", e);
                    }
                };

                tokio::time::sleep(Duration::from_secs(6)).await;
            }
        }
        Command::SweepPump { wallet_path } => {
            let keypair =
                Keypair::read_from_file(wallet_path).expect("read wallet");
            info!("Wallet: {}", keypair.pubkey());
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let pump_tokens = pump::get_tokens_held(&keypair.pubkey()).await?;
            for pump_token in pump_tokens {
                let mint = Pubkey::from_str(&pump_token.mint)?;
                let pump_accounts = pump::mint_to_pump_accounts(&mint).await?;
                if pump_token.balance > 0 {
                    // double-check balance of ata in order not to send a
                    // transaction bound to revert
                    let ata = spl_associated_token_account::get_associated_token_address(
                        &keypair.pubkey(),
                        &mint,
                    );
                    let actual_balance = rpc_client
                        .get_token_account_balance(&ata)
                        .await?
                        .amount
                        .parse::<u64>()?;
                    if actual_balance > 0 {
                        info!(
                            "Selling {} of {}",
                            actual_balance, pump_token.mint
                        );
                        pump::sell_pump_token(
                            &keypair,
                            &rpc_client,
                            pump_accounts,
                            pump_token.balance,
                        )
                        .await?;
                    }
                }
            }
        }
        Command::SnipePump { only_listen } => {
            info!("Pump snipe let's go");
            pump::snipe_pump(only_listen.unwrap_or(false)).await?;
        }
        Command::BuyPumpToken { mint: _ } => {
            // pump::buy_pump_token(Pubkey::from_str(&mint)?).await?;
            // return unimplemented err
            return Err("Unimplemented".into());
        }
        Command::GenerateCustomAddress { prefixes } => {
            let found_flag = Arc::new(AtomicBool::new(false));
            // note that this will only spawn as many workers as the runtime allows
            // defaults to num vcpus
            let workers: Vec<_> = (0..10)
                .map(|_| {
                    let prefixes = prefixes.clone();
                    let found_flag = Arc::clone(&found_flag);
                    tokio::spawn(async move {
                        address::generate_custom_sol_address(
                            prefixes, found_flag,
                        )
                        .await;
                    })
                })
                .collect();
            for worker in workers {
                worker.await?;
            }
        }
        Command::Ata { mint } => {
            let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .expect("read wallet");
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
            let pubsub_client = nonblocking::pubsub_client::PubsubClient::new(
                env("WS_URL").as_str(),
            )
            .await?;
            seller::get_spl_balance_stream(&pubsub_client, &ata).await?;
        }
        Command::Checks { signature } => {
            let (ok, checklist) = checker::run_checks(signature).await?;
            println!("ok? {}, {:?}", ok, checklist);
        }
        Command::Blockhash {} => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            for _ in 0..3 {
                let start = std::time::Instant::now();
                let res = rpc_client.get_latest_blockhash().await?;
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
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let new_pool = tx_parser::parse_new_pool(
                &Provider::get_tx(&rpc_client, signature.as_str()).await?,
            )?;
            println!("{:?}", new_pool);
        }
        Command::TopHolders { mint } => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let mint = Pubkey::from_str(mint.as_str()).unwrap();
            let (_, ok, _) =
                buyer::check_top_holders(&mint, &rpc_client, false).await?;
            info!("Top holders check passed: {}", ok);
        }
        Command::ListenForSolPooled { amm_pool } => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let pubsub_client = nonblocking::pubsub_client::PubsubClient::new(
                env("WS_URL").as_str(),
            )
            .await?;
            buyer::listen_for_sol_pooled(
                &Pubkey::from_str(amm_pool.as_str())?,
                &rpc_client,
                &pubsub_client,
            )
            .await?;
        }
        Command::ListenForBurn { amm_pool } => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let pubsub_client = nonblocking::pubsub_client::PubsubClient::new(
                env("WS_URL").as_str(),
            )
            .await?;
            buyer::listen_for_burn(
                &Pubkey::from_str(amm_pool.as_str())?,
                &rpc_client,
                &pubsub_client,
            )
            .await?;
        }
        Command::TrackPosition { amm_pool, owner } => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let amm_pool = Pubkey::from_str(amm_pool.as_str())
                .expect("amm pool is a valid pubkey");

            let amm_program = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;
            // load amm keys
            let amm_keys = amm::utils::load_amm_keys(
                &rpc_client,
                &amm_program,
                &amm_pool,
            )
            .await?;
            info!("{:?}", amm_keys);
            // load market keys
            let market_keys = amm::openbook::get_keys_for_market(
                &rpc_client,
                &amm_keys.market_program,
                &amm_keys.market,
            )
            .await?;
            info!("{:?}", market_keys);
            if market_keys.coin_mint.to_string()
                != constants::SOLANA_PROGRAM_ID.to_string()
                && market_keys.pc_mint.to_string()
                    != constants::SOLANA_PROGRAM_ID.to_string()
            {
                error!("pool is not against solana");
                return Ok(());
            }
            let coin_mint_is_sol = market_keys.coin_mint.to_string()
                == constants::SOLANA_PROGRAM_ID.to_string();
            let owner_balance = Provider::get_spl_balance(
                &rpc_client,
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
                    &rpc_client,
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
        Command::MonitorMempool {} => {
            let auth = Keypair::read_from_file(env("AUTH_KEYPAIR_PATH"))
                .expect("read auth keypair");
            let mut searcher_client =
                get_searcher_client(&env("BLOCK_ENGINE_URL"), &Arc::new(auth))
                    .await
                    .expect("makes searcher client");
            let res = searcher_client
                .subscribe_mempool(MempoolSubscription::default())
                .await?;
            info!("{:?}", res);
        }
        Command::MonitorLeaders {} => {
            let regions = vec![
                "frankfurt".to_string(),
                "amsterdam".to_string(),
                "tokyo".to_string(),
                "ny".to_string(),
                "slc".to_string(),
            ];
            let auth = Arc::new(
                Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap(),
            );
            let mut searcher_client =
                get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                    .await
                    .expect("makes searcher client");
            for region in regions {
                let res = searcher_client
                    .get_next_scheduled_leader(NextScheduledLeaderRequest {
                        regions: vec![region.clone()],
                    })
                    .await
                    .unwrap();
                let res = res.into_inner();
                println!(
                    "{}: at {} (in {} slots)",
                    res.next_leader_region,
                    res.next_leader_slot,
                    res.next_leader_slot - res.current_slot
                );
            }

            drop(searcher_client);
        }
        Command::MonitorSlots {} => {
            Listener::new(env("WS_URL").to_string()).slot_subscribe()?;
            let auth = Arc::new(
                Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap(),
            );
            let mut searcher_client =
                get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                    .await
                    .expect("makes searcher client");
            let _ = searcher_client
                .subscribe_mempool(MempoolSubscription::default())
                .await;
        }
        Command::BenchRPC { rpc_url } => rpc::eval_rpc(rpc_url.as_str()),
        Command::PriorityFee {} => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            println!(
                "{:?}",
                rpc_client
                    .get_recent_prioritization_fees(
                        vec![constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY]
                            .as_slice()
                    )
                    .await
            );
        }
        Command::Price { amm_pool } => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let pubsub_client = nonblocking::pubsub_client::PubsubClient::new(
                env("WS_URL").as_str(),
            )
            .await?;
            let amm_pool = Pubkey::from_str(amm_pool.as_str())?;
            seller::listen_price(&amm_pool, &rpc_client, &pubsub_client)
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
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let raydium = Raydium::new();
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
            if dex.unwrap_or_else(|| "".to_string()) == "raydium" {
                let amm_pool_id =
                    Pubkey::from_str(amm_pool_id.unwrap().as_str())?;
                let input_token_mint = Pubkey::from_str(input_mint.as_str())?;
                let output_token_mint =
                    Pubkey::from_str(output_mint.as_str())?;
                let slippage_bps = slippage.unwrap_or(800) as u64; // 8%
                let wallet = Keypair::read_from_file(path)?;
                info!("Wallet: {}", wallet.pubkey());
                info!(
                    "Balance (lamports): {}",
                    Provider::get_balance(&rpc_client, &wallet.pubkey())
                        .await?
                );
                let amount_specified = if amount.is_some() {
                    amount.unwrap() as u64
                } else {
                    Provider::get_spl_balance(
                        &rpc_client,
                        &wallet.pubkey(),
                        &input_token_mint,
                    )
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
                        rpc_client,
                        confirmed: yes.unwrap_or(false),
                        no_sanity: true,
                    })
                    .await?;
                return Ok(());
            }
            let keypair = Keypair::read_from_file(&path)?;
            if let Some(amount) = amount {
                let quote = Jupiter::fetch_quote(
                    &input_mint,
                    &output_mint,
                    amount as u64,
                    slippage.unwrap_or(75),
                )
                .await?;
                Jupiter::swap(quote, &keypair).await?;
            }
            let duration = start.elapsed();
            info!("Time elapsed: {:?}", duration);
            return Ok(());
        }
        Command::Wallet {} => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let path = match app.args.keypair_path {
                Some(path) => path,
                None => {
                    std::env::var("HOME").expect("HOME is set")
                        + "/.config/solana/id.json"
                }
            };
            let keypair = Keypair::read_from_file(&path)?;

            info!("Pubkey: {}", keypair.pubkey());
            let balance =
                Provider::get_balance(&rpc_client, &keypair.pubkey()).await?;
            info!("Balance: {} lamports", balance);
        }
        Command::Tx { signature } => {
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let tx = Provider::get_tx(&rpc_client, signature.as_str()).await?;
            info!("Tx: {}", serde_json::to_string_pretty(&tx)?);
            let mint = tx_parser::parse_mint(&tx)?;
            let pricing = Provider::get_pricing(&mint).await?;
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
            run_listener(worker_count as usize, buffer_size as usize).await?;
            return Ok(());
        }
    }
    Ok(())
}

pub async fn run_listener(
    worker_count: usize,
    buffer_size: usize,
) -> Result<(), Box<dyn Error>> {
    // let blocklist = vec![];
    let listener = Listener::new(env("WS_URL"));
    let (
        transactions_received,
        transactions_processed,
        requests_sent,
        registry,
    ) = prometheus::setup_metrics();

    // Start the metrics server
    let metrics_server = tokio::spawn(async move {
        prometheus::run_metrics_server(registry).await;
    });

    let (mut subs, recv) = listener.logs_subscribe()?; // Subscribe to logs

    let (tx, rx) =
        tokio::sync::mpsc::channel::<Response<RpcLogsResponse>>(buffer_size);
    let rx = Arc::new(Mutex::new(rx));

    // Worker tasks, increase in prod to way more, talking min 30-50
    let pool: Vec<_> = (0..worker_count)
        .map(|_| {
            let rx = Arc::clone(&rx);
            let rpc_client = RpcClient::new(env("RPC_URL"));
            let transactions_processed = transactions_processed.clone();
            let requests_sent = requests_sent.clone();
            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(Duration::from_millis(100)); // 10 requests per second
                while let Some(log) = rx.lock().await.recv().await {
                    interval.tick().await; // Rate limiting
                    let tx = {
                        match Provider::get_tx(
                            &rpc_client,
                            &log.value.signature,
                        )
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
                    requests_sent.inc();
                    let lamports =
                        tx_parser::parse_notional(&tx).ok().unwrap();
                    let sol_notional = util::lamports_to_sol(lamports);
                    transactions_processed.inc();
                    if sol_notional < 10. {
                        continue;
                    }
                    info!(
                        "https://solscan.io/tx/{}: {} SOL",
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

    Ok(())
}
