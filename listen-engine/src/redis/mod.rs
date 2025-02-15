use std::sync::Arc;

use crate::Engine;
use anyhow::Result;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, error};

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

pub struct RedisSubscriber {
    client: redis::Client,
    tx: broadcast::Sender<PriceUpdate>,
    engine: Arc<Engine>,
}

impl RedisSubscriber {
    pub fn new(redis_url: &str, engine: Arc<Engine>) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let (tx, _) = broadcast::channel(200);
        Ok(Self { client, tx, engine })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PriceUpdate> {
        self.tx.subscribe()
    }

    pub async fn start_listening(&self, channel: &str) -> Result<()> {
        let conn = self.client.get_async_connection().await?;
        debug!("Subscribing to Redis channel: {}", channel);

        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(channel).await?;
        let tx = self.tx.clone();
        let engine = self.engine.clone();

        tokio::spawn(async move {
            let mut msg_stream = pubsub.on_message();

            while let Some(msg) = msg_stream.next().await {
                match msg.get_payload::<String>() {
                    Ok(payload) => {
                        match serde_json::from_str::<PriceUpdate>(&payload) {
                            Ok(update) => {
                                // Forward price update to the engine
                                if let Err(e) = engine
                                    .handle_price_update(&update.pubkey, update.price)
                                    .await
                                {
                                    error!("Failed to handle price update: {}", e);
                                }
                                let _ = tx.send(update);
                            }
                            Err(e) => {
                                error!("Failed to parse price update: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get message payload: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

pub async fn create_redis_subscriber(
    redis_url: &str,
    engine: Arc<crate::Engine>,
) -> Result<Arc<RedisSubscriber>> {
    let subscriber = RedisSubscriber::new(redis_url, engine)?;
    subscriber.start_listening("price_updates").await?;

    Ok(Arc::new(subscriber))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;

    #[tokio::test]
    async fn test_redis_subscriber() {
        let engine = Arc::new(Engine::from_env().unwrap());
        let subscriber = create_redis_subscriber("redis://localhost:6379", engine)
            .await
            .unwrap();

        let mut sub = subscriber.subscribe();
        let msg = sub.recv().await.unwrap();
        assert!(!msg.pubkey.is_empty());
        assert!(msg.price > 0.0);
    }
}
