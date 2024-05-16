use crate::{buyer, constants, provider::Provider, tx_parser, util};
use dotenv_codegen::dotenv;
use futures_util::StreamExt;
use jito_searcher_client::get_searcher_client;
use log::info;
use serde_json::json;
use solana_client::rpc_pubsub::RpcLogsResponse;
use solana_client::rpc_response::{Response, RpcLogsResponse};
use solana_client::{
    nonblocking,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::EncodableKey;
use solana_sdk::signature::Keypair;
use std::{error::Error, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub async fn snipe(
    amount: u64,
    slippage: u64,
    worker_count: i32,
    buffer_size: i32,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Response<RpcLogsResponse>>(
        buffer_size as usize,
    );
    let rx = Arc::new(Mutex::new(rx));
    let listener = tokio::spawn(async move {
        let client =
            nonblocking::pubsub_client::PubsubClient::new(dotenv!("WS_URL"))
                .await
                .expect("pubsub client async");
        let (mut notifications, unsub) = client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![
                    constants::FEE_PROGRAM_ID.to_string(),
                ]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                },
            )
            .await
            .expect("subscribe to logs");
        info!("Listening for LP events");
        while let Some(log) = notifications.next().await {
            if log.value.err.is_none() {
                info!("passing log {}", log.value.signature);
                tx.send(log).await.expect("send log");
            }
        }
        unsub().await;
    });
    let buyer_pool: Vec<_> = (0..worker_count as usize)
        .map(|_| {
            let provider = Provider::new(dotenv!("RPC_URL").to_string());
            let wallet = Keypair::read_from_file(dotenv!("FUND_KEYPAIR_PATH"))
                .expect("read fund keypair");
            let auth = Arc::new(
                Keypair::read_from_file(dotenv!("AUTH_KEYPAIR_PATH"))
                    .expect("read auth keypair"),
            );
            let rx = Arc::clone(&rx);
            let auth = Arc::clone(&auth);
            tokio::spawn(async move {
                let mut searcher_client =
                    get_searcher_client(dotenv!("BLOCK_ENGINE_URL"), &auth)
                        .await
                        .expect("makes searcher client");
                while let Some(log) = rx.lock().await.recv().await {
                    let start = tokio::time::Instant::now();
                    let txn = provider.get_tx(&log.value.signature).unwrap();
                    info!("took {:?} to get tx", start.elapsed());
                    let new_pool_info = tx_parser::parse_new_pool(&txn)
                        .expect("parse pool info");
                    info!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "slot": log.context.slot,
                            "input": new_pool_info.input_mint.to_string(),
                            "output": new_pool_info.output_mint.to_string(),
                            "pool": new_pool_info.amm_pool_id.to_string(),
                            "amount": util::lamports_to_sol(amount),
                            "amm_pool": new_pool_info.amm_pool_id.to_string(),
                        }))
                        .expect("serialize pool info")
                    );
                    buyer::handle_new_pair(
                        new_pool_info,
                        amount,
                        slippage,
                        &wallet,
                        &provider,
                        &mut searcher_client,
                    )
                    .await
                    .expect("handle new pair");
                }
            })
        })
        .collect();
    listener.await?;
    for buyer in buyer_pool {
        buyer.await?;
    }
    return Ok(());
}
