use anyhow::{Context, Result};
use bb8_redis::{bb8, redis::cmd, RedisConnectionManager};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, info};

use crate::metadata::TokenMetadata;
use crate::price::Price;

/// internal impl
#[async_trait::async_trait]
pub trait KVStore {
    fn new(redis_url: &str) -> Result<Self>
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
}

#[async_trait::async_trait]
pub trait KVStoreExt: KVStore {
    async fn get_metadata(&self, mint: &str) -> Result<Option<TokenMetadata>>;
    async fn insert_metadata(&self, metadata: &TokenMetadata) -> Result<()>;
    async fn has_metadata(&self, mint: &str) -> Result<bool>;
    async fn get_price(
        &self,
        coin_mint: &str,
        pc_mint: &str,
    ) -> Result<Option<Price>>;
    async fn insert_price(&self, price: &Price) -> Result<()>;
}

pub struct RedisKVStore {
    pool: bb8::Pool<RedisConnectionManager>,
}

#[async_trait::async_trait]
impl KVStore for RedisKVStore {
    fn new(redis_url: &str) -> Result<Self> {
        let manager = RedisConnectionManager::new(redis_url)
            .context("Failed to create Redis connection manager")?;
        let pool = bb8::Pool::builder()
            .max_size(100)
            .min_idle(Some(20))
            .connection_timeout(std::time::Duration::from_secs(10))
            .idle_timeout(Some(std::time::Duration::from_secs(300)))
            // .max_lifetime(Some(std::time::Duration::from_secs(3600)))
            .build_unchecked(manager);
        info!("Connected to Redis at {}", redis_url);
        Ok(Self { pool })
    }

    async fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get connection from pool")?;

        let value: Option<String> = cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .context("Failed to get key")?;

        debug!(key, "redis get ok");

        return match value {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)?;
                Ok(Some(value))
            }
            None => Ok(None),
        };
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get connection from pool")?;
        let json_str = serde_json::to_string(value)?;
        let _: () = cmd("SET")
            .arg(key)
            .arg(json_str)
            .query_async(&mut *conn)
            .await
            .context("Failed to set key")?;
        debug!(key, "redis set ok");
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get connection from pool")?;
        let exists: bool = cmd("EXISTS")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .context("Failed to query exists")?;
        debug!(key, exists, "redis exists ok");
        Ok(exists)
    }
}

impl RedisKVStore {
    pub fn make_price_key(&self, coin_mint: &str, pc_mint: &str) -> String {
        format!("solana:price:{}:{}", coin_mint, pc_mint)
    }

    pub fn make_metadata_key(&self, mint: &str) -> String {
        format!("solana:metadata:{}", mint)
    }
}

#[async_trait::async_trait]
impl KVStoreExt for RedisKVStore {
    async fn insert_price(&self, price: &Price) -> Result<()> {
        let key = self.make_price_key(&price.coin_mint, &price.pc_mint);
        self.set(&key, price).await
    }

    async fn get_price(
        &self,
        coin_mint: &str,
        pc_mint: &str,
    ) -> Result<Option<Price>> {
        let key = self.make_price_key(coin_mint, pc_mint);
        self.get(&key).await
    }

    async fn insert_metadata(&self, metadata: &TokenMetadata) -> Result<()> {
        let key = self.make_metadata_key(&metadata.mint);
        self.set(&key, metadata).await
    }

    async fn get_metadata(&self, mint: &str) -> Result<Option<TokenMetadata>> {
        let key = self.make_metadata_key(mint);
        self.get(&key).await
    }

    async fn has_metadata(&self, mint: &str) -> Result<bool> {
        let key = self.make_metadata_key(mint);
        self.exists(&key).await
    }
}
