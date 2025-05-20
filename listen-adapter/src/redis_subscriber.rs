use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::broadcast;
use tracing::{debug, error};

pub struct RedisSubscriber {
    client: redis::Client,
    price_tx: broadcast::Sender<String>,
    transaction_tx: broadcast::Sender<String>,
}

impl RedisSubscriber {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let (price_tx, _) = broadcast::channel(200);
        let (transaction_tx, _) = broadcast::channel(200);
        Ok(Self {
            client,
            price_tx,
            transaction_tx,
        })
    }

    pub fn subscribe_prices(&self) -> broadcast::Receiver<String> {
        self.price_tx.subscribe()
    }

    pub fn subscribe_transactions(&self) -> broadcast::Receiver<String> {
        self.transaction_tx.subscribe()
    }

    pub async fn start_listening(&self) -> Result<()> {
        let conn = self.client.get_async_connection().await?;
        debug!("Subscribing to Redis channels");

        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe("price_updates").await?;
        pubsub.subscribe("transaction_updates").await?;

        let price_tx = self.price_tx.clone();
        let transaction_tx = self.transaction_tx.clone();

        tokio::spawn(async move {
            let mut msg_stream = pubsub.on_message();

            while let Some(msg) = msg_stream.next().await {
                let channel: String = msg.get_channel_name().to_string();
                match msg.get_payload::<String>() {
                    Ok(payload) => match channel.as_str() {
                        "price_updates" => {
                            let _ = price_tx.send(payload);
                        }
                        "transaction_updates" => {
                            let _ = transaction_tx.send(payload);
                        }
                        _ => {
                            error!("Unknown channel: {}", channel);
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

pub async fn create_redis_subscriber(redis_url: &str) -> anyhow::Result<Arc<RedisSubscriber>> {
    let subscriber = RedisSubscriber::new(redis_url)?;
    subscriber.start_listening().await?;
    Ok(Arc::new(subscriber))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_subscriber() {
        let subscriber = create_redis_subscriber("redis://localhost:6379")
            .await
            .unwrap();

        let mut price_sub = subscriber.subscribe_prices();
        let mut tx_sub = subscriber.subscribe_transactions();

        // You can add test messages here using a separate Redis client
        // For now just testing the setup works
        assert!(subscriber.price_tx.receiver_count() > 0);
        assert!(subscriber.transaction_tx.receiver_count() > 0);
    }
}
