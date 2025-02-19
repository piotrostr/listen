use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bincode;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::VersionedTransaction;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// RPC response types
#[derive(Deserialize, Debug, Serialize)]
struct RpcResponse {
    result: BlockhashResponse,
}

#[derive(Deserialize, Debug, Serialize)]
struct BlockhashResponse {
    value: BlockhashValue,
}

#[derive(Deserialize, Debug, Serialize)]
struct BlockhashValue {
    blockhash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BlockhashCacheError {
    #[error("[BlockhashCache] Failed to fetch blockhash")]
    FetchError(#[from] reqwest::Error),

    #[error("[BlockhashCache] Failed to parse blockhash")]
    ParseError(#[from] bs58::decode::Error),

    #[error("[BlockhashCache] Failed to convert blockhash to Hash")]
    HashConversionError,
}

pub static BLOCKHASH_CACHE: Lazy<BlockhashCache> = Lazy::new(|| {
    BlockhashCache::new(&std::env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set"))
});

pub struct BlockhashCache {
    blockhash: Arc<RwLock<String>>,
    client: reqwest::Client,
    rpc_url: String,
}

impl BlockhashCache {
    pub fn new(rpc_url: &str) -> Self {
        let client = reqwest::Client::new();
        let blockhash = Arc::new(RwLock::new(String::default()));
        let rpc_url = rpc_url.to_string();

        let cache = Self {
            blockhash,
            client,
            rpc_url,
        };
        cache.start_update_task();
        cache
    }

    fn start_update_task(&self) {
        let blockhash = self.blockhash.clone();
        let client = self.client.clone();
        let rpc_url = self.rpc_url.clone();

        tokio::spawn(async move {
            loop {
                match Self::fetch_blockhash(&client, &rpc_url).await {
                    Ok(new_blockhash) => {
                        let mut hash_writer = blockhash.write().await;
                        *hash_writer = new_blockhash;
                        drop(hash_writer);
                    }
                    Err(err) => {
                        tracing::error!("Failed to fetch blockhash: {}", err);
                    }
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }

    async fn fetch_blockhash(client: &reqwest::Client, rpc_url: &str) -> Result<String> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestBlockhash",
            "params": [{"commitment": "finalized"}]
        });

        let response = client.post(rpc_url).json(&request_body).send().await?;
        let response_json: RpcResponse = response.json().await?;

        // Convert the base58 string to our Hash type
        let blockhash = response_json.result.value.blockhash;
        Ok(blockhash)
    }

    pub async fn get_blockhash(&self) -> Result<String> {
        let current_hash = self.blockhash.read().await.clone();

        // If we have a valid blockhash (not default), return it
        if current_hash != String::default() {
            return Ok(current_hash);
        }

        // If we don't have a valid blockhash yet, fetch it immediately
        Self::fetch_blockhash(&self.client, &self.rpc_url).await
    }
}

pub fn inject_blockhash_into_encoded_tx(base64_tx: &str, blockhash: &str) -> Result<String> {
    // Decode base64 transaction into bytes
    let tx_bytes = STANDARD.decode(base64_tx)?;

    // Deserialize into VersionedTransaction
    let mut transaction = bincode::deserialize::<VersionedTransaction>(&tx_bytes)?;

    // Convert blockhash string to Hash
    let blockhash = solana_sdk::hash::Hash::from_str(&blockhash)
        .map_err(|_| anyhow::anyhow!("Invalid blockhash format"))?;

    // Update the blockhash
    match &mut transaction.message {
        solana_sdk::message::VersionedMessage::Legacy(message) => {
            message.recent_blockhash = blockhash;
        }
        solana_sdk::message::VersionedMessage::V0(message) => {
            message.recent_blockhash = blockhash;
        }
    }

    // Serialize back to bytes
    let updated_tx_bytes = bincode::serialize(&transaction)?;

    // Encode to base64
    Ok(STANDARD.encode(updated_tx_bytes))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_blockhash_cache() {
        dotenv::dotenv().ok();
        let blockhash = super::BLOCKHASH_CACHE.get_blockhash().await.unwrap();
        assert_ne!(blockhash, String::default());
    }
}
