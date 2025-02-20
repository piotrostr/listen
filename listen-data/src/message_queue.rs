use crate::util::create_redis_pool;
use anyhow::{Context, Result};
use bb8_redis::{bb8, RedisConnectionManager};
use tracing::info;

use crate::price::PriceUpdate;

#[async_trait::async_trait]
pub trait MessageQueue: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn publish_price_update(
        &self,
        price_update: PriceUpdate,
    ) -> Result<(), Self::Error>;
}

// Redis implementation of MessageQueue
#[derive(Debug)]
pub struct RedisMessageQueue {
    pool: bb8::Pool<RedisConnectionManager>,
}

impl RedisMessageQueue {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let pool = create_redis_pool(redis_url).await?;
        info!("Connected to Redis message queue at {}", redis_url);
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl MessageQueue for RedisMessageQueue {
    type Error = redis::RedisError;

    async fn publish_price_update(
        &self,
        price_update: PriceUpdate,
    ) -> Result<(), Self::Error> {
        let mut conn = self
            .pool
            .get()
            .await
            .context(format!(
                "Failed to get Redis connection: {:#?}",
                self.pool.state().statistics
            ))
            .map_err(|e| {
                redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "Failed to get Redis connection",
                    e.to_string(),
                ))
            })?;
        let payload = serde_json::to_string(&price_update).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Serialization error",
                e.to_string(),
            ))
        })?;

        redis::cmd("PUBLISH")
            .arg("price_updates")
            .arg(payload)
            .query_async(&mut *conn)
            .await
    }
}
