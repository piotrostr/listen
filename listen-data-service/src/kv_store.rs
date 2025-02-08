use anyhow::Result;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::metadata::TokenMetadata;

#[async_trait::async_trait]
pub trait KVStore {
    fn new() -> Self
    where
        Self: Sized;
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>>;
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub(crate) coin_price: f64,
    pub(crate) pc_price: f64,
    pub(crate) coin_mint: String,
    pub(crate) pc_mint: String,
}

pub struct RedisKVStore {
    client: redis::Client,
}

#[async_trait::async_trait]
impl KVStore for RedisKVStore {
    fn new() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to connect to Redis");
        Self { client }
    }

    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let json_str = serde_json::to_string(value)?;
        let _: () = conn.set(key, json_str).await?;
        Ok(())
    }
}

impl RedisKVStore {
    pub fn make_price_key(price: &Price) -> String {
        format!("solana:{}:{}", price.coin_mint, price.pc_mint)
    }
    pub fn make_metadata_key(metadata: &TokenMetadata) -> String {
        format!("solana:{}", metadata.mint)
    }

    pub async fn insert_price(&self, price: &Price) -> Result<()> {
        let key = Self::make_price_key(price);
        self.set(&key, price).await
    }

    pub async fn get_price(&self, coin_mint: &str, pc_mint: &str) -> Result<Option<Price>> {
        let key = format!("solana:{}:{}", coin_mint, pc_mint);
        self.get(&key).await
    }

    pub async fn insert_metadata(&self, metadata: &TokenMetadata) -> Result<()> {
        let key = Self::make_metadata_key(metadata);
        self.set(&key, metadata).await
    }

    pub async fn get_metadata(&self, mint: &str) -> Result<Option<TokenMetadata>> {
        let key = format!("solana:{}", mint);
        self.get(&key).await
    }
}
