use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::data::Candlestick; // Added for timestamp conversion

// Define the structure for the token information we want to return
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtTokenMetadata {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: Option<u32>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub websites: Option<Vec<String>>,
    pub chain_id: u64,
    pub discord_url: Option<String>,
    pub telegram_handle: Option<String>,
    pub twitter_handle: Option<String>,
}

// this is optional, might be useful at some point
// #[derive(Deserialize, Debug)]
// struct GTHolders {
//     count: u64,
//     distribution_percentage: GTDistributionPercentage,
// }
//
// #[derive(Deserialize, Debug)]
// struct GTDistributionPercentage {
//     top_10: f32,
// }

// Structs for deserializing GeckoTerminal Token Info API response
#[derive(Deserialize, Debug)]
struct GTTokenInfoAttributes {
    name: String,
    address: String,
    symbol: String,
    decimals: Option<u32>,
    image_url: Option<String>,
    description: Option<String>,
    websites: Option<Vec<String>>,
    discord_url: Option<String>,
    telegram_handle: Option<String>,
    twitter_handle: Option<String>,
    // gt_score: Option<f32>,
    // holders: Option<GTHolders>, // Add other fields if needed from the API response schema
}

#[derive(Deserialize, Debug)]
struct GTTokenInfoData {
    attributes: GTTokenInfoAttributes,
}

#[derive(Deserialize, Debug)]
struct GTTokenInfoResponse {
    data: GTTokenInfoData,
}

// Structs for deserializing GeckoTerminal OHLCV API response
#[derive(Deserialize, Debug)]
struct GTOhlcvAttributes {
    // [timestamp(s), open, high, low, close, volume]
    ohlcv_list: Vec<[Value; 6]>,
}

#[derive(Deserialize, Debug)]
struct GTOhlcvData {
    attributes: GTOhlcvAttributes,
}

#[derive(Deserialize, Debug)]
struct GTCandlesticksResponse {
    data: GTOhlcvData,
}

// Helper function to map chain ID (u64) to GeckoTerminal network string
fn map_chain_id_to_network(chain_id: u64) -> Result<&'static str> {
    match chain_id {
        1 => Ok("eth"),
        56 => Ok("bsc"),
        42161 => Ok("arbitrum"),
        8453 => Ok("base"),
        _ => Err(anyhow!("Unsupported chain ID: {}", chain_id)),
    }
}

// Helper function to map interval string to GeckoTerminal timeframe and aggregate
// Example interval formats: "1m", "5m", "15m", "1h", "4h", "1d"
fn map_interval_to_params(
    interval: &str,
) -> Result<(&'static str, &'static str)> {
    match interval {
        "1m" => Ok(("minute", "1")),
        "5m" => Ok(("minute", "5")),
        "15m" => Ok(("minute", "15")),
        "1h" => Ok(("hour", "1")),
        "4h" => Ok(("hour", "4")),
        "1d" => Ok(("day", "1")),
        _ => Err(anyhow!("Unsupported interval format: {}", interval)),
    }
}

pub struct EvmFallback {
    client: Client,
    base_url: String,
    api_version: String,
}

impl EvmFallback {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.geckoterminal.com/api/v2".to_string(),
            api_version: "20230302".to_string(), // As specified in the API docs
        }
    }

    pub async fn fetch_token_info(
        &self,
        address: &str,
        chain_id: u64,
    ) -> Result<GtTokenMetadata> {
        let network = map_chain_id_to_network(chain_id)?;
        let url = format!(
            "{}/networks/{}/tokens/{}/info",
            self.base_url, network, address
        );

        let response = self
            .client
            .get(&url)
            .header(
                "Accept",
                format!("application/json;version={}", self.api_version),
            )
            .send()
            .await
            .context(format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow!(
                "GeckoTerminal API request failed for token info ({}): {} - {}",
                url,
                status,
                error_text
            ));
        }

        let gt_token_info_resp =
            response.json::<GTTokenInfoResponse>().await.context(
                "Failed to deserialize GeckoTerminal token info response",
            )?;

        let attributes = gt_token_info_resp.data.attributes;

        let token_info = GtTokenMetadata {
            address: attributes.address,
            name: attributes.name,
            symbol: attributes.symbol,
            decimals: attributes.decimals,
            image_url: attributes.image_url,
            description: attributes.description,
            websites: attributes.websites,
            discord_url: attributes.discord_url,
            telegram_handle: attributes.telegram_handle,
            twitter_handle: attributes.twitter_handle,
            chain_id,
        };

        Ok(token_info)
    }

    pub async fn fetch_candlesticks(
        &self,
        pool_address: &str,
        chain_id: u64,
        interval: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Candlestick>> {
        let network = map_chain_id_to_network(chain_id)?;
        let (timeframe, aggregate) = map_interval_to_params(interval)?;

        let mut url = format!(
            "{}/networks/{}/pools/{}/ohlcv/{}",
            self.base_url, network, pool_address, timeframe
        );

        // Build query parameters
        let mut query_params = HashMap::new();
        query_params.insert("aggregate".to_string(), aggregate.to_string());

        if let Some(limit) = limit {
            // API max limit is 1000
            query_params
                .insert("limit".to_string(), limit.min(1000).to_string());
        } else {
            // Default limit from API docs is 100
            query_params.insert("limit".to_string(), "100".to_string());
        }

        let query_string = query_params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&");

        if !query_string.is_empty() {
            url.push('?');
            url.push_str(&query_string);
        }

        let response = self
            .client
            .get(&url)
            .header(
                "Accept",
                format!("application/json;version={}", self.api_version),
            )
            .send()
            .await
            .context(format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow!(
                "GeckoTerminal API request failed for OHLCV ({}): {} - {}",
                url,
                status,
                error_text
            ));
        }

        let gt_candlesticks_resp = response
            .json::<GTCandlesticksResponse>()
            .await
            .context("Failed to deserialize GeckoTerminal OHLCV response")?;

        // Convert GTOhlcvData to Vec<Candlestick>
        let candlesticks = gt_candlesticks_resp
            .data
            .attributes
            .ohlcv_list
            .into_iter()
            .filter_map(|item| {
                // Expecting [timestamp, open, high, low, close, volume]
                if item.len() != 6 {
                    eprintln!(
                        "Warning: Received malformed OHLCV item: {:?}",
                        item
                    );
                    return None;
                }
                let timestamp_val = item[0].as_u64()?;
                let open_val = item[1].as_f64()?;
                let high_val = item[2].as_f64()?;
                let low_val = item[3].as_f64()?;
                let close_val = item[4].as_f64()?;
                let volume_val = item[5].as_f64()?;

                Some(Candlestick {
                    timestamp: timestamp_val,
                    open: open_val,
                    high: high_val,
                    low: low_val,
                    close: close_val,
                    volume: volume_val,
                })
            })
            .collect::<Vec<Candlestick>>();

        Ok(candlesticks)
    }
}

// Make the test module public if needed for external test runners or keep private
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_token_info_evm() {
        let fallback = EvmFallback::new();
        // Use a known token on Ethereum (chain_id 1) e.g., PEPE
        let address = "0x6982508145454Ce325dDbE47a25d4ec3d2311933";
        let chain_id = 1; // Ethereum

        let result = fallback.fetch_token_info(address, chain_id).await;

        println!("Token Info Result: {:?}", result);
        assert!(result.is_ok());
        let token_info = result.unwrap();
        assert_eq!(token_info.address.to_lowercase(), address.to_lowercase());
        assert_eq!(token_info.symbol, "PEPE");
        assert_eq!(token_info.chain_id, chain_id);
        assert!(token_info.decimals.is_some());
    }

    #[tokio::test]
    async fn test_get_candlesticks_evm() {
        let fallback = EvmFallback::new();
        // Use a known pool on Ethereum (chain_id 1) e.g., PEPE/WETH
        let pool_address = "0xA43fe16908251ee70EF74718545e4FE6C5cCEc9f"; // PEPE/WETH 0.3% on Uniswap V3
        let chain_id = 1; // Ethereum
        let interval = "15m";
        let limit = Some(10);

        let result = fallback
            .fetch_candlesticks(pool_address, chain_id, interval, limit)
            .await;

        println!("Candlesticks Result: {:?}", result);
        assert!(result.is_ok());
        let candlesticks = result.unwrap();
        assert!(!candlesticks.is_empty());
        // Check if limit is respected (API might return fewer if less data exists)
        assert!(candlesticks.len() <= limit.unwrap_or(100));

        // Basic check on the first candlestick
        if let Some(first_candle) = candlesticks.first() {
            println!("First candlestick: {:?}", first_candle);
            assert!(first_candle.open > 0.0);
            assert!(first_candle.high > 0.0);
            assert!(first_candle.low > 0.0);
            assert!(first_candle.close > 0.0);
            assert!(first_candle.volume >= 0.0); // Volume can be 0
            assert!(first_candle.timestamp > 0); // Timestamp should be positive
        }
    }

    #[tokio::test]
    async fn test_unsupported_chain_id() {
        let fallback = EvmFallback::new();
        let address = "0x0000000000000000000000000000000000000000";
        let chain_id = 99999; // Unsupported chain ID

        let result_info = fallback.fetch_token_info(address, chain_id).await;
        assert!(result_info.is_err());
        assert!(result_info
            .unwrap_err()
            .to_string()
            .contains("Unsupported chain ID"));

        let result_candles = fallback
            .fetch_candlesticks(address, chain_id, "15m", Some(10))
            .await;
        assert!(result_candles.is_err());
        assert!(result_candles
            .unwrap_err()
            .to_string()
            .contains("Unsupported chain ID"));
    }

    #[tokio::test]
    async fn test_unsupported_interval() {
        let fallback = EvmFallback::new();
        let pool_address = "0xA43fe16908251ee70EF74718545e4FE6C5cCEc9f";
        let chain_id = 1;
        let interval = "1y"; // Unsupported interval

        let result_candles = fallback
            .fetch_candlesticks(pool_address, chain_id, interval, Some(10))
            .await;
        assert!(result_candles.is_err());
        assert!(result_candles
            .unwrap_err()
            .to_string()
            .contains("Unsupported interval format"));
    }

    #[tokio::test]
    async fn test_nonexistent_token() {
        let fallback = EvmFallback::new();
        // Use a clearly invalid address
        let address = "0x000000000000000000000000000000000000dead";
        let chain_id = 1; // Ethereum

        let result = fallback.fetch_token_info(address, chain_id).await;
        // Expecting a 404 or similar error from the API, mapped to anyhow::Error
        assert!(result.is_err());
        println!("Nonexistent token error: {:?}", result.unwrap_err());
    }

    #[tokio::test]
    async fn test_nonexistent_pool() {
        let fallback = EvmFallback::new();
        let pool_address = "0x000000000000000000000000000000000000dead";
        let chain_id = 1;
        let interval = "15m";
        let limit = Some(10);

        let result = fallback
            .fetch_candlesticks(pool_address, chain_id, interval, limit)
            .await;
        // Expecting a 404 or similar error from the API
        assert!(result.is_err());
        println!("Nonexistent pool error: {:?}", result.unwrap_err());
    }
}
