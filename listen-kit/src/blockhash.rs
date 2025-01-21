use crate::util::env;
use lazy_static::lazy_static;
use solana_client::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

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

    // Start the background task to update the blockhash
    fn start_update_task(&self) {
        let blockhash = self.blockhash.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            loop {
                match client.get_latest_blockhash() {
                    Ok(new_blockhash) => {
                        let mut hash_writer = blockhash.write().await;
                        *hash_writer = new_blockhash;
                        drop(hash_writer);
                    }
                    Err(err) => {
                        eprintln!("Failed to fetch blockhash: {}", err);
                    }
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }

    // Get the current blockhash
    pub async fn get_blockhash(&self) -> Hash {
        *self.blockhash.read().await
    }
}

lazy_static! {
    pub static ref BLOCKHASH_CACHE: BlockhashCache =
        BlockhashCache::new(&env("RPC_URL"));
}
