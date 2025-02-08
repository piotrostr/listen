use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;

use crate::kv_store::{KVStore, RedisKVStore};

pub fn make_rpc_client() -> Result<RpcClient> {
    let rpc_client = RpcClient::new(std::env::var("RPC_URL")?);
    Ok(rpc_client)
}

pub fn make_kv_store() -> Result<Arc<RedisKVStore>> {
    let kv_store = RedisKVStore::new();
    Ok(Arc::new(kv_store))
}
