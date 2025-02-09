use crate::price::PriceUpdate;

#[async_trait::async_trait]
pub trait MessageQueue: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn publish_price_update(&self, price_update: PriceUpdate) -> Result<(), Self::Error>;
}

// Redis implementation of MessageQueue
pub struct RedisMessageQueue {
    client: redis::Client,
}

impl RedisMessageQueue {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl MessageQueue for RedisMessageQueue {
    type Error = redis::RedisError;

    async fn publish_price_update(&self, price_update: PriceUpdate) -> Result<(), Self::Error> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
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
            .query_async(&mut conn)
            .await
    }
}
