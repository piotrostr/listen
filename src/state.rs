use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ServiceState {
    pub wallet: Arc<Mutex<Keypair>>,
    pub rpc_client: Arc<RpcClient>,
    pub latest_blockhash: Arc<Mutex<Hash>>,
}
