use log::{debug, error};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

pub async fn update_latest_blockhash(
    rpc_client: Arc<RpcClient>,
    latest_blockhash: Arc<Mutex<Hash>>,
) {
    let mut interval = interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        match rpc_client.get_latest_blockhash().await {
            Ok(new_blockhash) => {
                let mut blockhash = latest_blockhash.lock().await;
                *blockhash = new_blockhash;
                debug!("Updated latest blockhash: {}", new_blockhash);
            }
            Err(e) => {
                error!("Failed to get latest blockhash: {}", e);
            }
        }
    }
}
