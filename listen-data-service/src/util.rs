use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::{fs::File, io::BufWriter, sync::Arc};

use crate::{
    db::{ClickhouseDb, Database},
    kv_store::{KVStore, RedisKVStore},
    message_queue::RedisMessageQueue,
};

pub fn is_local() -> bool {
    std::env::var("LOCAL").is_ok()
}

pub fn make_rpc_client() -> Result<RpcClient> {
    let rpc_client = RpcClient::new(must_get_env("RPC_URL"));
    Ok(rpc_client)
}

pub fn make_kv_store() -> Result<Arc<RedisKVStore>> {
    match is_local() {
        true => {
            let kv_store = RedisKVStore::new("redis://localhost:6379")?;
            Ok(Arc::new(kv_store))
        }
        false => {
            let kv_store =
                RedisKVStore::new(must_get_env("REDIS_URL").as_str())?;
            Ok(Arc::new(kv_store))
        }
    }
}

pub fn make_message_queue() -> Result<Arc<RedisMessageQueue>> {
    match is_local() {
        true => {
            let message_queue =
                RedisMessageQueue::new("redis://localhost:6379")?;
            Ok(Arc::new(message_queue))
        }
        false => {
            let message_queue =
                RedisMessageQueue::new(must_get_env("REDIS_URL").as_str())?;
            Ok(Arc::new(message_queue))
        }
    }
}

pub async fn make_db() -> Result<Arc<ClickhouseDb>> {
    let mut db = match is_local() {
        true => ClickhouseDb::new(
            "http://localhost:8123",
            "default",
            "default",
            "default",
        ),
        false => ClickhouseDb::new(
            must_get_env("CLICKHOUSE_URL").as_str(),
            must_get_env("CLICKHOUSE_USER").as_str(),
            must_get_env("CLICKHOUSE_PASSWORD").as_str(),
            must_get_env("CLICKHOUSE_DATABASE").as_str(),
        ),
    };
    db.initialize().await?;
    Ok(Arc::new(db))
}

pub fn write_json(data: &str, file_name: &str) -> Result<()> {
    let file = File::create(file_name)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, data)?;
    Ok(())
}

pub fn round_to_decimals(x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x * y).round() / y
}

pub async fn get_jup_price(mint: String) -> Result<f64> {
    let url = format!(
        "https://api.jup.ag/price/v2?ids={}&vsToken=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        mint
    );

    let response = reqwest::get(&url).await?;
    let json: serde_json::Value = response.json().await?;

    // Extract price from response
    let price = json["data"][&mint]["price"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse price"))?;

    let price = price.parse::<f64>()?;

    Ok(price)
}

pub fn must_get_env(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => panic!("{} must be set", key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_jup_price() {
        let price = get_jup_price(
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string(),
        )
        .await;
        assert!(price.is_ok());
        assert!(price.unwrap() > 0.0);
    }
}
