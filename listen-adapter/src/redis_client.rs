use anyhow::{Context, Result};
use bb8_redis::{bb8, redis::cmd, RedisConnectionManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

pub struct RedisClient {
    pool: bb8::Pool<RedisConnectionManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MplTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub ipfs_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SplTokenMetadata {
    pub mint_authority: Option<String>,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenMetadata {
    pub mint: String,
    pub mpl: MplTokenMetadata,
    pub spl: SplTokenMetadata,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let manager = RedisConnectionManager::new(redis_url)?;
        let pool = bb8::Pool::builder().max_size(10).build(manager).await?;

        Ok(Self { pool })
    }

    fn make_metadata_key(&self, mint: &str) -> String {
        format!("solana:metadata:{}", mint)
    }
    fn make_price_key(&self, mint: &str) -> String {
        format!("solana:price:{}", mint)
    }

    pub async fn get_price(&self, mint: &str) -> Result<serde_json::Value> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get Redis connection")?;

        let key = self.make_price_key(mint);
        let value: Option<String> = cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .with_context(|| format!("Failed to get price for mint: {}", mint))?;

        match value {
            Some(price_str) => serde_json::from_str(&price_str)
                .with_context(|| format!("Failed to deserialize price for mint: {}", mint)),
            None => {
                debug!(mint, "No price found");
                Err(anyhow::anyhow!("No price found for mint: {}", mint))
            }
        }
    }

    pub async fn get_metadata(&self, mint: &str) -> Result<Option<TokenMetadata>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get Redis connection")?;

        let key = self.make_metadata_key(mint);
        let value: Option<String> = cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .with_context(|| format!("Failed to get metadata for mint: {}", mint))?;

        match value {
            Some(json_str) => {
                debug!(mint, "Found metadata");
                serde_json::from_str(&json_str)
                    .with_context(|| format!("Failed to deserialize metadata for mint: {}", mint))
                    .map(Some)
            }
            None => {
                debug!(mint, "No metadata found");
                Ok(None)
            }
        }
    }

    fn make_chat_key(&self, chat_id: &str) -> String {
        format!("chats:shared:{}", chat_id)
    }

    pub async fn get_chat(&self, chat_id: &str) -> Result<Option<serde_json::Value>> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get Redis connection")?;

        let key = self.make_chat_key(chat_id);
        let value: Option<String> = cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .with_context(|| format!("Failed to get chat: {}", chat_id))?;

        match value {
            Some(json_str) => serde_json::from_str(&json_str)
                .with_context(|| format!("Failed to deserialize chat: {}", chat_id)),
            None => {
                debug!(chat_id, "No chat found");
                Ok(None)
            }
        }
    }

    pub async fn save_chat(&self, chat_id: &str, chat: &serde_json::Value) -> Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .context("Failed to get Redis connection")?;

        let key = self.make_chat_key(chat_id);
        let _: () = cmd("SET")
            .arg(key)
            .arg(serde_json::to_string(chat)?)
            .query_async(&mut *conn)
            .await
            .with_context(|| format!("Failed to save chat: {}", chat_id))?;

        Ok(())
    }
}

pub async fn make_redis_client() -> Result<Arc<RedisClient>> {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = RedisClient::new(&redis_url).await?;
    Ok(Arc::new(client))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_metadata() {
        let client = make_redis_client().await.unwrap();
        let metadata: Option<TokenMetadata> = client
            .get_metadata("9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump")
            .await
            .unwrap();
        println!("Metadata: {:?}", metadata);
    }
}
