use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

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
    SendError(String),

    #[error("[RedisSubscriber] JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("[RedisSubscriber] Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
}

pub struct RedisSubscriber {
    client: redis::Client,
    tx: mpsc::Sender<PriceUpdate>,
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl RedisSubscriber {
    pub fn new(
        redis_url: &str,
        tx: mpsc::Sender<PriceUpdate>,
    ) -> Result<Self, RedisSubscriberError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            tx,
            task_handle: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start_listening(&self) -> Result<(), RedisSubscriberError> {
        let conn = self.client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe("price_updates").await?;
        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            let mut msg_stream = pubsub.on_message();
            let mut consecutive_errors = 0;
            let last_message_time = Instant::now();
            let mut metrics_interval = tokio::time::interval(Duration::from_secs(1));

            while let Some(msg) = msg_stream.next().await {
                metrics::counter!("redis_messages_received", 1);

                if metrics_interval.tick().await.elapsed().is_zero() {
                    metrics::gauge!(
                        "redis_subscriber_last_message_age_seconds",
                        last_message_time.elapsed().as_secs_f64()
                    );

                    metrics::gauge!("price_update_channel_capacity", tx.capacity() as f64);
                }

                match msg.get_payload::<String>() {
                    Ok(payload) => {
                        consecutive_errors = 0;
                        match serde_json::from_str::<PriceUpdate>(&payload) {
                            Ok(update) => {
                                metrics::counter!("price_updates_parsed", 1);
                                tracing::debug!(
                                    "Processing price update: asset={}, price={}, timestamp={}",
                                    update.name,
                                    update.price,
                                    update.timestamp
                                );

                                match tx.try_send(update) {
                                    Ok(_) => {
                                        metrics::counter!("price_updates_sent", 1);
                                    }
                                    Err(e) => match e {
                                        tokio::sync::mpsc::error::TrySendError::Full(update) => {
                                            metrics::counter!("price_update_channel_full", 1);
                                            if let Err(e) = tx.blocking_send(update) {
                                                error!(
                                                    "Failed to send price update (blocking): {}",
                                                    e
                                                );
                                                metrics::counter!("price_updates_send_errors", 1);
                                                if tx.is_closed() {
                                                    error!(
                                                        "Channel closed, stopping subscriber task"
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                        tokio::sync::mpsc::error::TrySendError::Closed(e) => {
                                            error!("Failed to send price update: {}", e.signature);
                                            metrics::counter!("price_updates_send_errors", 1);
                                            if tx.is_closed() {
                                                error!("Channel closed, stopping subscriber task");
                                                break;
                                            }
                                        }
                                    },
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse price update: {}", e);
                                metrics::counter!("price_updates_parse_errors", 1);
                                consecutive_errors += 1;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get message payload: {}", e);
                        metrics::counter!("redis_payload_errors", 1);
                        consecutive_errors += 1;
                    }
                }

                if consecutive_errors > 10 {
                    error!("Too many consecutive errors, reconnecting...");
                    metrics::counter!("redis_reconnection_attempts", 1);
                    break;
                }
            }

            metrics::counter!("redis_subscriber_exits", 1);
        });

        let mut task_handle = self.task_handle.lock().await;
        *task_handle = Some(handle);

        Ok(())
    }

    pub async fn check_health(&self) -> bool {
        let task_handle = self.task_handle.lock().await;
        if let Some(handle) = &*task_handle {
            !handle.is_finished()
        } else {
            false
        }
    }

    pub async fn ensure_running(&self) -> Result<(), RedisSubscriberError> {
        let task_handle = self.task_handle.lock().await;
        if let Some(handle) = &*task_handle {
            if handle.is_finished() {
                drop(task_handle);
                self.start_listening().await?;
            }
        } else {
            drop(task_handle);
            self.start_listening().await?;
        }
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
