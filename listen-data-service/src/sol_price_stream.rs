use anyhow::Result;
use futures_util::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info};
use url::Url;

#[derive(Debug, Deserialize)]
struct TradeData {
    p: String,
}

#[derive(Debug, Clone)]
pub struct SolPriceCache {
    price: Arc<RwLock<f64>>,
}

impl SolPriceCache {
    pub fn new() -> Self {
        Self {
            price: Arc::new(RwLock::new(0.0)),
        }
    }

    pub async fn get_price(&self) -> f64 {
        *self.price.read().await
    }

    pub async fn start_price_stream(self) -> Result<()> {
        let url = Url::parse("wss://stream.binance.com:9443/ws/solusdt@trade")?;
        let (ws_stream, _) = connect_async(url).await?;
        info!("WebSocket connected to Binance SOL/USDT stream");

        let (_, mut read) = ws_stream.split();

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => match serde_json::from_str::<TradeData>(&text) {
                    Ok(trade) => {
                        if let Ok(new_price) = trade.p.parse::<f64>() {
                            *self.price.write().await = new_price;
                        }
                    }
                    Err(e) => error!("Error parsing JSON: {}", e),
                },
                Ok(Message::Ping(_)) => {}
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_sol_price_cache() {
        let price_cache = SolPriceCache::new();
        let price_cache_clone = price_cache.clone();

        // Spawn the price stream in a separate task
        tokio::spawn(async move {
            if let Err(e) = price_cache.start_price_stream().await {
                error!("Error in price stream: {}", e);
            }
        });

        // Wait a bit for the first price update
        sleep(Duration::from_secs(2)).await;

        // Get the price
        let price = price_cache_clone.get_price().await;
        assert!(price > 0.0, "Price should be greater than 0");
        info!("Current SOL price: ${:.3}", price);
    }
}
