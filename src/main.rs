use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use clap::Parser;
use listen::{tx_parser, util, Listener, Provider};
use solana_client::rpc_response::{Response, RpcLogsResponse};
use tokio::sync::Mutex;

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
        let mint = tx_parser::parse_mint(&tx)?;
        let pricing = provider.get_pricing(&mint).await?;
        println!("Pricing: {:?}", pricing);

        let swap = tx_parser::parse_swap(&tx)?;
        println!("Swap: {}", serde_json::to_string_pretty(&swap)?);

        let sol_notional =
            listen::util::lamports_to_sol(swap.quote_amount as u64);

        let usd_notional = sol_notional * sol_price;

        println!("{} ({} USD)", sol_notional, usd_notional);
        // TODO
        // include transaction signature for confirmation
        // refactor into notional and run in prod with rate limit (10 at a time)

        return Ok(());
    }

    if args.listen {
        let listener = Listener::new(args.ws_url);
        let (mut subs, recv) = listener.logs_subscribe()?; // Subscribe to logs

        let (tx, rx) =
            tokio::sync::mpsc::channel::<Response<RpcLogsResponse>>(64); // Create a channel with a buffer size of 32
        let rx = Arc::new(Mutex::new(rx));

        // Worker tasks, increase in prod to way more, talking min 30-50
        let workers: Vec<_> = (0..3)
            .map(|_| {
                let rx = Arc::clone(&rx);
                let provider = Provider::new(util::must_get_env("RPC_URL"));
                tokio::spawn(async move {
                    let mut rx = rx.lock().await;
                    while let Some(log) = rx.recv().await {
                        let tx = provider.get_tx(&log.value.signature).unwrap();
                        let changes =
                            tx_parser::parse_swap_from_balances_change(&tx);
                        println!(
                            "{}: {}",
                            &log.value.signature,
                            serde_json::to_string_pretty(&changes).unwrap()
                        );
                    }
                })
            })
            .collect();

        // Log receiving task
        let log_receiver = tokio::spawn(async move {
            while let Ok(log) = recv.recv_timeout(Duration::from_secs(1)) {
                if log.value.err.is_some() {
                    continue; // Skip error logs
                }
                match tx.send(log.clone()).await {
                    Err(e) => println!("Failed to send log: {}", e),
                    Ok(_) => {
                        println!("passing on log, slot: {}", log.context.slot);
                        println!(
                            "https://solana.fm/tx/{}",
                            log.value.signature
                        );
                    }
                }
            }
            drop(tx);
            subs.shutdown().unwrap(); // Shutdown subscription on exit
        });

        // Await all tasks
        log_receiver.await?;
        for worker in workers {
            worker.await?;
        }
    }

    Ok(())
}
