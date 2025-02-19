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
    FetchError(reqwest::Error),

    #[error("[BlockhashCache] Failed to parse blockhash")]
    ParseError(reqwest::Error),

    #[error("[BlockhashCache] Failed to convert blockhash to Hash")]
    HashConversionError,
}

pub static BLOCKHASH_CACHE: Lazy<BlockhashCache> = Lazy::new(|| {
    BlockhashCache::new(&std::env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set"))
});

pub struct BlockhashCache {
    blockhash: Arc<RwLock<solana_sdk::hash::Hash>>,
    client: reqwest::Client,
    rpc_url: String,
}

impl BlockhashCache {
    pub fn new(rpc_url: &str) -> Self {
        let client = reqwest::Client::new();
        let blockhash = Arc::new(RwLock::new(solana_sdk::hash::Hash::default()));
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

    async fn fetch_blockhash(
        client: &reqwest::Client,
        rpc_url: &str,
    ) -> Result<solana_sdk::hash::Hash, BlockhashCacheError> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestBlockhash",
            "params": [{"commitment": "finalized"}]
        });

        let response = client
            .post(rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(BlockhashCacheError::FetchError)?;
        let response_json: RpcResponse = response
            .json()
            .await
            .map_err(BlockhashCacheError::ParseError)?;

        // Convert the base58 string to our Hash type
        let blockhash = response_json.result.value.blockhash;
        let hash = solana_sdk::hash::Hash::from_str(&blockhash)
            .map_err(|_| BlockhashCacheError::HashConversionError)?;
        Ok(hash)
    }

    pub async fn get_blockhash(&self) -> Result<solana_sdk::hash::Hash, BlockhashCacheError> {
        let current_hash = self.blockhash.read().await.clone();

        // If we have a valid blockhash (not default), return it
        if current_hash != solana_sdk::hash::Hash::default() {
            return Ok(current_hash);
        }

        // If we don't have a valid blockhash yet, fetch it immediately
        Self::fetch_blockhash(&self.client, &self.rpc_url).await
    }
}

pub fn inject_blockhash_into_encoded_tx(base64_tx: &str, blockhash: &str) -> Result<String> {
    // Decode base64 transaction into bytes
    let tx_bytes = STANDARD.decode(base64_tx)?;

    // Try to deserialize as VersionedTransaction first
    let updated_tx_bytes = match bincode::deserialize::<VersionedTransaction>(&tx_bytes) {
        Ok(mut transaction) => {
            // Handle versioned transaction
            let blockhash = solana_sdk::hash::Hash::from_str(&blockhash)
                .map_err(|_| anyhow::anyhow!("Invalid blockhash format"))?;

            match &mut transaction.message {
                solana_sdk::message::VersionedMessage::Legacy(message) => {
                    message.recent_blockhash = blockhash;
                }
                solana_sdk::message::VersionedMessage::V0(message) => {
                    message.recent_blockhash = blockhash;
                }
            }
            bincode::serialize(&transaction)?
        }
        Err(_) => {
            // Try as standard Transaction
            let mut transaction =
                bincode::deserialize::<solana_sdk::transaction::Transaction>(&tx_bytes)?;

            let blockhash = solana_sdk::hash::Hash::from_str(&blockhash)
                .map_err(|_| anyhow::anyhow!("Invalid blockhash format"))?;

            transaction.message.recent_blockhash = blockhash;
            bincode::serialize(&transaction)?
        }
    };

    // Encode to base64
    Ok(STANDARD.encode(updated_tx_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_blockhash_cache() {
        dotenv::dotenv().ok();
        let blockhash = super::BLOCKHASH_CACHE.get_blockhash().await.unwrap();
        assert_ne!(blockhash, solana_sdk::hash::Hash::default());
    }

    #[test]
    fn test_inject_blockhash_standard_transaction() {
        // Create a simple standard transaction
        let payer = Keypair::new();
        let to = Pubkey::new_unique();
        let instruction = system_instruction::transfer(&payer.pubkey(), &to, 1000);
        let message = Message::new(&[instruction], Some(&payer.pubkey()));
        let tx = Transaction::new_unsigned(message);

        // Serialize and encode the transaction
        let tx_bytes = bincode::serialize(&tx).unwrap();
        let base64_tx = STANDARD.encode(&tx_bytes);

        // Test injecting a new blockhash
        let new_blockhash = "CkqVVMoo6LUqzqKSQVFNL4Yxv3TXyTh1NQxTSG2Z9gTq";
        let result = inject_blockhash_into_encoded_tx(&base64_tx, new_blockhash).unwrap();

        // Decode and deserialize the result
        let updated_bytes = STANDARD.decode(result).unwrap();
        let updated_tx: Transaction = bincode::deserialize(&updated_bytes).unwrap();

        assert_eq!(
            updated_tx.message.recent_blockhash.to_string(),
            new_blockhash
        );
    }
}
