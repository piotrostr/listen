use crate::solana::util::env;
use anyhow::{anyhow, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use once_cell::sync::Lazy;

pub static BLOCKHASH_CACHE: Lazy<BlockhashCache> =
    Lazy::new(|| BlockhashCache::new(&env("SOLANA_RPC_URL")));

pub struct BlockhashCache {
    blockhash: Arc<RwLock<Hash>>,
    client: Arc<RpcClient>,
}

impl BlockhashCache {
    pub fn new(rpc_url: &str) -> Self {
        let client = Arc::new(RpcClient::new(rpc_url.to_string()));
        let blockhash = Arc::new(RwLock::new(Hash::default()));

        let cache = Self { blockhash, client };
        cache.start_update_task();
        cache
    }

    fn start_update_task(&self) {
        let blockhash = self.blockhash.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            loop {
                match client
                    .get_latest_blockhash_with_commitment(
                        CommitmentConfig::finalized(),
                    )
                    .await
                {
                    Ok(res) => {
                        let new_blockhash = res.0;
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

    pub async fn get_blockhash(&self) -> Result<Hash> {
        let current_hash = *self.blockhash.read().await;

        // If we have a valid blockhash (not default), return it
        if current_hash != Hash::default() {
            return Ok(current_hash);
        }

        // If we don't have a valid blockhash yet, fetch it immediately
        match self
            .client
            .get_latest_blockhash_with_commitment(
                CommitmentConfig::finalized(),
            )
            .await
        {
            Ok(res) => {
                let new_blockhash = res.0;
                let mut hash_writer = self.blockhash.write().await;
                *hash_writer = new_blockhash;
                Ok(new_blockhash)
            }
            Err(err) => {
                Err(anyhow!("Failed to fetch initial blockhash: {}", err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_blockhash_cache() {
        let blockhash = super::BLOCKHASH_CACHE.get_blockhash().await.unwrap();
        assert_ne!(blockhash, Default::default());
    }
}
