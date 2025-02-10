use anyhow::Result;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};

use tracing::{debug, info};

use crate::metadata::TokenMetadata;
use crate::price::Price;

#[async_trait::async_trait]
pub trait KVStore {
    fn new(redis_url: &str) -> Self
    where
        Self: Sized;
    async fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>>;
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn get_metadata(&self, mint: &str) -> Result<Option<TokenMetadata>>;
}

pub struct RedisKVStore {
    client: redis::Client,
}

#[async_trait::async_trait]
impl KVStore for RedisKVStore {
    fn new(redis_url: &str) -> Self {
        let client =
            redis::Client::open(redis_url).expect("Failed to connect to Redis");
        info!("Connected to Redis at {}", redis_url);
        Self { client }
    }

    async fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(key).await?;
        debug!(key, "redis get ok");

        match value {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let json_str = serde_json::to_string(value)?;
        let _: () = conn.set(key, json_str).await?;
        debug!(key, "redis set ok");
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let exists: bool =
            redis::cmd("EXISTS").arg(key).query_async(&mut conn).await?;
        debug!(key, exists, "redis exists ok");
        Ok(exists)
    }

    async fn get_metadata(&self, mint: &str) -> Result<Option<TokenMetadata>> {
        let key = format!("solana:{}", mint);
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let data: Option<String> = conn.get(&key).await?;

        match data {
            Some(json_str) => {
                let metadata: TokenMetadata = serde_json::from_str(&json_str)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }
}

impl RedisKVStore {
    pub fn make_price_key(price: &Price) -> String {
        format!("solana:{}:{}", price.coin_mint, price.pc_mint)
    }
    pub fn make_metadata_key(mint: &str) -> String {
        format!("solana:{}", mint)
    }

    pub async fn insert_price(&self, price: &Price) -> Result<()> {
        let key = Self::make_price_key(price);
        self.set(&key, price).await
    }

    pub async fn get_price(
        &self,
        coin_mint: &str,
        pc_mint: &str,
    ) -> Result<Option<Price>> {
        let key = format!("solana:{}:{}", coin_mint, pc_mint);
        self.get(&key).await
    }

    pub async fn insert_metadata(
        &self,
        metadata: &TokenMetadata,
    ) -> Result<()> {
        let key = Self::make_metadata_key(&metadata.mint);
        self.set(&key, metadata).await
    }

    pub async fn get_metadata(
        &self,
        mint: &str,
    ) -> Result<Option<TokenMetadata>> {
        let key = format!("solana:{}", mint);
        self.get(&key).await
    }

    pub async fn has_metadata(&self, mint: &str) -> Result<bool> {
        self.exists(&Self::make_metadata_key(mint)).await
    }
}
