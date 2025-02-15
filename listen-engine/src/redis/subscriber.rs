use std::sync::Arc;

use anyhow::Result;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceUpdate {
    pub name: String,
    pub pubkey: String,
    pub price: f64,
    pub market_cap: f64,
    pub timestamp: u64,
    pub slot: u64,
    pub swap_amount: f64, // denoted as usd
    pub owner: String,
    pub signature: String,
    pub multi_hop: bool,
    pub is_buy: bool,
    pub is_pump: bool,
}

#[derive(Error, Debug)]
pub enum RedisSubscriberError {
    #[error("[RedisSubscriber] Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("[RedisSubscriber] Failed to send price update: {0}")]
    SendError(#[from] mpsc::error::SendError<PriceUpdate>),

    #[error("[RedisSubscriber] JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("[RedisSubscriber] Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
}

pub struct RedisSubscriber {
    client: redis::Client,
    tx: mpsc::Sender<PriceUpdate>,
}

impl RedisSubscriber {
    pub fn new(
        redis_url: &str,
        tx: mpsc::Sender<PriceUpdate>,
    ) -> Result<Self, RedisSubscriberError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client, tx })
    }

    pub async fn start_listening(&self) -> Result<(), RedisSubscriberError> {
        let conn = self.client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe("price_updates").await?;
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let mut msg_stream = pubsub.on_message();

            while let Some(msg) = msg_stream.next().await {
                match msg.get_payload::<String>() {
                    Ok(payload) => match serde_json::from_str::<PriceUpdate>(&payload) {
                        Ok(update) => {
                            if let Err(e) = tx.send(update).await {
                                error!("Failed to send price update: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse price update: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to get message payload: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

pub fn make_redis_subscriber(
    tx: mpsc::Sender<PriceUpdate>,
) -> Result<Arc<RedisSubscriber>, RedisSubscriberError> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let subscriber = RedisSubscriber::new(&redis_url, tx)?;
    Ok(Arc::new(subscriber))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_subscriber() {
        let (tx, mut rx) = mpsc::channel(100);
        let subscriber = make_redis_subscriber(tx).unwrap();

        subscriber.start_listening().await.unwrap();

        let msg = rx.recv().await.unwrap();
        assert!(!msg.pubkey.is_empty());
        assert!(msg.price > 0.0);
    }
}
