use anyhow::{anyhow, Result};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::{
    Encodable, EncodedTransaction, UiTransactionEncoding,
};
use std::cell::RefCell;
use std::str::FromStr;
use tracing::info;

use crate::solana::util::env;

#[derive(Debug, Deserialize)]
pub struct JitoResponse {
    pub jsonrpc: String,
    pub result: String,
    pub id: i64,
}

#[timed::timed(duration(printer = "info!"))]
pub async fn send_jito_tx(tx: Transaction) -> Result<String> {
    let client = reqwest::Client::new();

    let encoded_tx = match tx.encode(UiTransactionEncoding::Binary) {
        EncodedTransaction::LegacyBinary(b) => b,
        _ => return Err(anyhow!("Failed to encode transaction")),
    };

    let res = client
        .post("https://mainnet.block-engine.jito.wtf/api/v1/transactions")
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [encoded_tx]
        }))
        .send()
        .await
        .expect("send tx");

    let jito_response = res.json::<JitoResponse>().await.map_err(|e| {
        anyhow!("Failed to parse jito response: {}", e.to_string())
    })?;

    Ok(jito_response.result)
}

pub async fn send_tx_fallback(tx: &Transaction) -> Result<String> {
    let rpc_client = RpcClient::new(env("SOLANA_RPC_URL"));

    let signature = rpc_client
        .send_transaction_with_config(
            tx,
            RpcSendTransactionConfig {
                max_retries: Some(3),
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )
        .await
        .map_err(|e| {
            anyhow!("Failed to send transaction: {}", e.to_string())
        })?;

    tracing::info!(?signature, "send_tx_fallback");

    Ok(signature.to_string())
}

pub async fn send_tx(tx: &Transaction) -> Result<String> {
    if std::env::var("SKIP_SIMULATION").is_err() {
        let simres = RpcClient::new(env("SOLANA_RPC_URL"))
            .simulate_transaction_with_config(
                tx,
                RpcSimulateTransactionConfig {
                    replace_recent_blockhash: true,
                    ..RpcSimulateTransactionConfig::default()
                },
            )
            .await?;
        if simres.value.err.is_some() {
            return Err(anyhow!(
                "Transaction simulation failed: {:?}",
                simres
            ));
        }
    }

    let signature = send_jito_tx(tx.clone()).await;
    if let Ok(signature) = &signature {
        tracing::info!(?signature, "send_jito_tx");
    }
    match signature {
        Ok(signature) => Ok(signature),
        Err(e) => {
            let msg = e.to_string();
            tracing::warn!(?msg, "send_jito_tx");
            send_tx_fallback(tx).await
        }
    }
}

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

#[inline(always)]
pub fn fast_random_0_to_7() -> u8 {
    RNG.with(|rng| rng.borrow_mut().gen_range(0..8))
}

pub fn get_jito_tip_pubkey() -> Pubkey {
    const PUBKEYS: [&str; 8] = [
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
        "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
        "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
        "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
        "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
        "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
        "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
        "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
    ];
    let index = fast_random_0_to_7();
    Pubkey::from_str(PUBKEYS[index as usize]).expect("parse tip pubkey")
}

#[cfg(test)]
mod tests {
    #[test]
    fn bench_get_jito_tip_pubkey() {
        for _ in 0..100 {
            let start = std::time::Instant::now();
            let _ = super::get_jito_tip_pubkey();
            let elapsed = start.elapsed();
            tracing::info!(?elapsed, "bench_get_jito_tip_pubkey");
        }
    }
}
