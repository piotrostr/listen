use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use tokio::sync::broadcast;
use tracing::{debug, error};

pub struct RedisSubscriber {
    client: redis::Client,
    tx: broadcast::Sender<String>,
}

impl RedisSubscriber {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let (tx, _) = broadcast::channel(200);
        Ok(Self { client, tx })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub async fn start_listening(&self, channel: &str) -> Result<()> {
        let conn = self.client.get_async_connection().await?;
        debug!("Subscribing to Redis channel: {}", channel);

        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(channel).await?;
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let mut msg_stream = pubsub.on_message();

            while let Some(msg) = msg_stream.next().await {
                match msg.get_payload::<String>() {
                    Ok(payload) => {
                        let _ = tx.send(payload);
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

pub async fn create_redis_subscriber(redis_url: &str) -> anyhow::Result<Arc<RedisSubscriber>> {
    let subscriber = RedisSubscriber::new(redis_url)?;
    subscriber.start_listening("price_updates").await?;

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
        subscriber.start_listening("price_updates").await.unwrap();

        let mut sub = subscriber.subscribe();
        let msg = sub.recv().await.unwrap();
        assert!(msg != "");
    }
}
