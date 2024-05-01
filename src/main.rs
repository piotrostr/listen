use std::{sync::Arc, time::Duration};

use prometheus::{Encoder, IntCounter, Registry, TextEncoder};

use clap::Parser;
use listen::{tx_parser, util, Listener, Provider};
use solana_client::rpc_response::{Response, RpcLogsResponse};
use tokio::sync::Mutex;
use warp::Filter;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    signature: Option<String>,

    #[arg(short, long)]
    listen: bool,

    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    url: String,

    #[arg(short, long, default_value = "wss://api.mainnet-beta.solana.com")]
    ws_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 30th April, let's see how well this ages lol
    let sol_price = 135.;
    let args = Args::parse();

    if let Some(signature) = args.signature {
        let provider = Provider::new(args.url);
        let tx = provider.get_tx(&signature)?;
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

    if args.listen {
        let (transactions_received, transactions_processed, registry) =
            setup_metrics();

        // Start the metrics server
        let metrics_server = tokio::spawn(async move {
            run_metrics_server(registry).await;
        });

        let listener = Listener::new(args.ws_url);
        let (mut subs, recv) = listener.logs_subscribe()?; // Subscribe to logs

        let (tx, rx) =
            tokio::sync::mpsc::channel::<Response<RpcLogsResponse>>(1280); // Create a channel with a buffer size of 32
        let rx = Arc::new(Mutex::new(rx));

        // Worker tasks, increase in prod to way more, talking min 30-50
        let workers: Vec<_> = (0..1)
            .map(|_| {
                let rx = Arc::clone(&rx);
                let provider = Provider::new(util::must_get_env("RPC_URL"));
                let transactions_processed = transactions_processed.clone();
                tokio::spawn(async move {
                    while let Some(log) = rx.lock().await.recv().await {
                        let tx = provider.get_tx(&log.value.signature).unwrap();
                        let lamports =
                            tx_parser::parse_notional(&tx).ok().unwrap();
                        let sol_notional = util::lamports_to_sol(lamports);
                        transactions_processed.inc();
                        if sol_notional < 10. {
                            continue;
                        }
                        println!(
                            "https://solana.fm/tx/{}: {} SOL",
                            &log.value.signature,
                            sol_notional,
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
        for worker in workers {
            worker.await?;
        }
    }

    Ok(())
}

static TRANSACTIONS_RECEIVED: &str = "transactions_received";
static TRANSACTIONS_PROCESSED: &str = "transactions_processed";

fn setup_metrics() -> (Arc<IntCounter>, Arc<IntCounter>, Registry) {
    let registry = Registry::new();
    let transactions_received = IntCounter::new(
        TRANSACTIONS_RECEIVED,
        "Total number of transactions received",
    )
    .unwrap();
    let transactions_processed = IntCounter::new(
        TRANSACTIONS_PROCESSED,
        "Total number of transactions processed",
    )
    .unwrap();

    registry
        .register(Box::new(transactions_received.clone()))
        .unwrap();
    registry
        .register(Box::new(transactions_processed.clone()))
        .unwrap();

    (
        Arc::new(transactions_received),
        Arc::new(transactions_processed),
        registry,
    )
}

async fn run_metrics_server(registry: Registry) {
    // Metrics endpoint
    let metrics_route = warp::path!("metrics").map(move || {
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        warp::reply::with_header(buffer, "Content-Type", encoder.format_type())
    });

    println!("Metrics server running on {}", 3030);
    warp::serve(metrics_route).run(([127, 0, 0, 1], 3030)).await;
}
