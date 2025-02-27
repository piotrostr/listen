use anyhow::{Context, Result};
use bb8_redis::{bb8, redis::cmd, RedisConnectionManager};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, info};

use crate::metadata::TokenMetadata;
use crate::price::PriceUpdate;
use crate::util::create_redis_pool;

#[derive(Debug, Clone)]
pub struct RedisKVStore {
    pool: bb8::Pool<RedisConnectionManager>,
}

impl RedisKVStore {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let pool = create_redis_pool(redis_url).await?;
        info!("Connected to Redis KV store at {}", redis_url);
        Ok(Self { pool })
    }

    pub async fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get Redis connection")?;

        let value: Option<String> = cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .with_context(|| {
                format!("Failed to execute GET for key: {}", key)
            })?;

        match value {
            Some(json_str) => serde_json::from_str(&json_str)
                .with_context(|| {
                    format!("Failed to deserialize value for key: {}", key)
                })
                .map(Some),
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<()> {
        let mut conn = self.pool.get().await.context(format!(
            "Failed to get Redis connection: {:#?}",
            self.pool.state().statistics
        ))?;
        let json_str = serde_json::to_string(value)?;
        let _: () = cmd("SET")
            .arg(key)
            .arg(json_str)
            .query_async(&mut *conn)
            .await
            .with_context(|| format!("Failed to set key: {}", key))?;
        debug!(key, "redis set ok");
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.pool.get().await.context(format!(
            "Failed to get Redis connection: {:#?}",
            self.pool.state().statistics
        ))?;
        let exists: bool = cmd("EXISTS")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .with_context(|| {
                format!("Failed to query exists for key: {}", key)
            })?;
        debug!(key, exists, "redis exists ok");
        Ok(exists)
    }

    fn make_price_key(&self, mint: &str) -> String {
        format!("solana:price:{}", mint)
    }

    fn make_metadata_key(&self, mint: &str) -> String {
        format!("solana:metadata:{}", mint)
    }

    pub async fn insert_price(&self, price: &PriceUpdate) -> Result<()> {
        let key = self.make_price_key(&price.pubkey);
        self.set(&key, price).await
    }

    pub async fn get_price(&self, pubkey: &str) -> Result<Option<PriceUpdate>> {
        let key = self.make_price_key(pubkey);
        self.get(&key).await
    }

    pub async fn insert_metadata(
        &self,
        metadata: &TokenMetadata,
    ) -> Result<()> {
        let key = self.make_metadata_key(&metadata.mint);
        self.set(&key, metadata).await
    }

    pub async fn get_metadata(
        &self,
        mint: &str,
    ) -> Result<Option<TokenMetadata>> {
        let key = self.make_metadata_key(mint);
        self.get(&key).await
    }

    pub async fn has_metadata(&self, mint: &str) -> Result<bool> {
        let key = self.make_metadata_key(mint);
        self.exists(&key).await
    }
}
