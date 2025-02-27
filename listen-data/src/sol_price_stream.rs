use crate::{
    kv_store::RedisKVStore,
    message_queue::{MessageQueue, RedisMessageQueue},
    price::PriceUpdate,
};
use anyhow::Result;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info};
use url::Url;

// Change the global cache to be just the price without Redis connections
pub static SOL_PRICE_CACHE: Lazy<Arc<RwLock<f64>>> =
    Lazy::new(|| Arc::new(RwLock::new(0.0)));

#[derive(Debug, Deserialize)]
struct TradeData {
    p: String,
}

#[derive(Debug, Deserialize)]
struct BinancePrice {
    price: String,
}

#[derive(Debug, Clone)]
pub struct SolPriceCache {
    price: Arc<RwLock<f64>>,
    message_queue: Option<Arc<RedisMessageQueue>>,
    kv_store: Option<Arc<RedisKVStore>>,
}

impl SolPriceCache {
    pub fn new(
        kv_store: Option<Arc<RedisKVStore>>,
        message_queue: Option<Arc<RedisMessageQueue>>,
    ) -> Self {
        Self {
            price: SOL_PRICE_CACHE.clone(), // Use the global price cache
            message_queue,
            kv_store,
        }
    }

    async fn publish_price_update(&self, new_price: f64) -> Result<()> {
        let price_update = PriceUpdate {
            name: "Solana".to_string(),
            pubkey: crate::constants::WSOL_MINT_KEY_STR.to_string(),
            price: new_price,
            market_cap: 0.0, // Could calculate if we had supply
            timestamp: Utc::now().timestamp() as u64,
            slot: 0,          // Not applicable for Binance price
            swap_amount: 0.0, // Not applicable
            owner: "binance".to_string(),
            signature: "binance_websocket".to_string(),
            multi_hop: false,
            is_buy: false,
            is_pump: false,
        };
        if let Some(kv_store) = &self.kv_store {
            kv_store.insert_price(&price_update).await?;
        }
        if let Some(mq) = &self.message_queue {
            mq.publish_price_update(price_update).await?;
        }
        Ok(())
    }

    pub async fn set_price(&self, price: f64) {
        *self.price.write().await = price;
    }

    pub async fn get_price(&self) -> f64 {
        let current_price = *self.price.read().await;
        if current_price == 0.0 {
            match self.fetch_rest_price().await {
                Ok(rest_price) => {
                    *self.price.write().await = rest_price;
                    rest_price
                }
                Err(e) => {
                    error!("Failed to fetch REST price: {}", e);
                    current_price
                }
            }
        } else {
            current_price
        }
    }

    async fn fetch_rest_price(&self) -> Result<f64> {
        let rest_url =
            "https://api.binance.com/api/v3/ticker/price?symbol=SOLUSDT";
        let response = reqwest::get(rest_url).await?;
        let price_data: BinancePrice = response.json().await?;
        price_data.price.parse::<f64>().map_err(Into::into)
    }

    pub async fn start_price_stream(&self) -> Result<()> {
        loop {
            info!("Connecting to Binance WebSocket...");
            match self.connect_and_stream().await {
                Ok(_) => {
                    info!("WebSocket stream ended gracefully");
                    // Optional delay before reconnecting on graceful close
                    tokio::time::sleep(tokio::time::Duration::from_secs(1))
                        .await;
                }
                Err(e) => {
                    error!("WebSocket stream error: {}. Reconnecting in 5 seconds...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5))
                        .await;
                }
            }
        }
    }

    async fn connect_and_stream(&self) -> Result<()> {
        let url = Url::parse("wss://fstream.binance.com/ws/solusdt@aggTrade")?;
        let (ws_stream, _) = connect_async(url).await?;
        let price_cache = self.clone();
        info!("WebSocket connected to Binance SOL/USDT futures stream");

        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));
        let write_clone = write.clone();

        // Spawn a task to handle pong responses
        let pong_task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                if let Err(e) =
                    write_clone.lock().await.send(Message::Pong(vec![])).await
                {
                    error!("Failed to send pong: {}", e);
                    break;
                }
            }
        });

        let result = async {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<TradeData>(&text) {
                            Ok(trade) => {
                                if let Ok(new_price) = trade.p.parse::<f64>() {
                                    let current_price =
                                        price_cache.get_price().await;
                                    if current_price != new_price {
                                        price_cache.set_price(new_price).await;
                                        let price_cache = price_cache.clone();
                                        tokio::spawn(async move {
                                            if let Err(e) = price_cache
                                                .publish_price_update(new_price)
                                                .await
                                            {
                                                error!("Failed to publish price update: {}", e);
                                            }
                                        });
                                    }
                                } else {
                                    error!("Failed to parse price: {}", text);
                                }
                            }
                            Err(e) => error!("Error parsing JSON: {}", e),
                        }
                    }
                    Ok(Message::Ping(_)) => {
                        if let Err(e) = write.lock().await.send(Message::Pong(vec![])).await {
                            error!("Failed to send pong: {}", e);
                            return Err(anyhow::anyhow!("Pong send error: {}", e));
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        info!("WebSocket closed by server: {:?}", frame);
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("WebSocket error: {}", e));
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        .await;

        // Cancel the pong task when the main loop exits
        pong_task.abort();
        result
    }
}

// Add a convenience function for getting the global price
pub async fn get_sol_price() -> f64 {
    *SOL_PRICE_CACHE.read().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_sol_price_cache() {
        let price_cache = SolPriceCache::new(None, None);
        let price_cache_clone = price_cache.clone();

        // Spawn the price stream in a separate task
        tokio::spawn(async move {
            if let Err(e) = price_cache.start_price_stream().await {
                error!("Error in price stream: {}", e);
            }
        });

        // Wait a bit for the first price update
        sleep(Duration::from_secs(2)).await;

        let price = price_cache_clone.get_price().await;
        info!("Current SOL price: ${:.3}", price);
        assert!(price > 0.0, "Price should be greater than 0");
    }

    #[tokio::test]
    async fn test_rest_fallback() {
        let price_cache = SolPriceCache::new(None, None);

        // Test initial state (should trigger REST fallback)
        let price = price_cache.get_price().await;
        info!("Initial SOL price from REST: ${:.3}", price);
        assert!(price > 0.0, "REST fallback price should be greater than 0");

        // Test that the price was cached
        let cached_price = *price_cache.price.read().await;
        assert_eq!(
            price, cached_price,
            "Price should be cached after REST call"
        );
    }
}
