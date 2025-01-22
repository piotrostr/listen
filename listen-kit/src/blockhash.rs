use crate::util::env;
use lazy_static::lazy_static;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock};

pub struct BlockhashCache {
    blockhash: Arc<RwLock<Hash>>,
    client: Arc<RpcClient>,
    initialized: Arc<Notify>,
}

impl BlockhashCache {
    pub fn new(rpc_url: &str) -> Self {
        let client = Arc::new(RpcClient::new(rpc_url.to_string()));
        let blockhash = Arc::new(RwLock::new(Hash::default()));
        let initialized = Arc::new(Notify::new());

        let cache = Self {
            blockhash,
            client,
            initialized,
        };
        cache.start_update_task();
        cache
    }

    fn start_update_task(&self) {
        let blockhash = self.blockhash.clone();
        let client = self.client.clone();
        let initialized = self.initialized.clone();

        tokio::spawn(async move {
            // First update
            if let Ok(new_blockhash) = client.get_latest_blockhash().await {
                let mut hash_writer = blockhash.write().await;
                *hash_writer = new_blockhash;
                drop(hash_writer);
                initialized.notify_one(); // Notify that initial fetch is complete
            }

            // Continuous updates
            loop {
                match client.get_latest_blockhash().await {
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

    pub async fn get_blockhash(&self) -> Hash {
        // Wait for the initial fetch to complete
        self.initialized.notified().await;

        *self.blockhash.read().await
    }
}

lazy_static! {
    pub static ref BLOCKHASH_CACHE: BlockhashCache =
        BlockhashCache::new(&env("RPC_URL"));
}
