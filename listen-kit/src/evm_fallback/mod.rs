use anyhow::{anyhow, Context, Result};
use std::env;

pub mod candlesticks;
pub mod find_pair;
pub mod token_info;
pub mod top_tokens;
pub mod top_tokens_by_category;
pub mod top_tokens_by_category_cmc;

pub struct EvmFallback {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

fn supported_chain_ids_map() -> serde_json::Value {
    serde_json::json!({
        "1": "eth",
        "56": "bsc",
        "42161": "arbitrum",
        "8453": "base",
        "480": "world-chain",
        "1151111081099710": "solana",
        "eip:1": "eth",
        "eip:56": "bsc",
        "eip:42161": "arbitrum",
        "eip:8453": "base",
        "eip:480": "world-chain",
        "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp": "solana",
    })
}

// Helper function to map chain ID (u64) to CoinGecko network string
pub fn map_chain_id_to_network(chain_id: String) -> Result<String> {
    let map = supported_chain_ids_map();
    map.get(&chain_id.to_string())
        .ok_or(anyhow!(
            "Unsupported chain ID: {}, supported chains: {}",
            chain_id,
            map
        ))
        .map(|s| s.as_str().unwrap_or_default().to_string())
}

pub fn validate_chain_id(chain_id: String) -> Result<()> {
    map_chain_id_to_network(chain_id)?;
    Ok(())
}

pub const SOLANA_CHAIN_ID: &str = "1151111081099710";

impl EvmFallback {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("GECKO_API_KEY")
            .context("GECKO_API_KEY environment variable not set")?;

        Ok(Self {
            client: reqwest::Client::new(),
            base_url: "https://pro-api.coingecko.com/api/v3/onchain"
                .to_string(),
            api_key,
        })
    }

    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://pro-api.coingecko.com/api/v3/onchain"
                .to_string(),
            api_key,
        }
    }
}

// Make the test module public if needed for external test runners or keep private
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_token_info_evm() {
        let fallback = EvmFallback::from_env()
            .expect("Failed to create EvmFallback from environment");
        // Use a known token on Ethereum (chain_id 1) e.g., PEPE
        let address = "0x6982508145454Ce325dDbE47a25d4ec3d2311933";
        let chain_id = "1".to_string(); // Ethereum

        let result =
            fallback.fetch_token_info(address, chain_id.clone()).await;

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
        let fallback = EvmFallback::from_env()
            .expect("Failed to create EvmFallback from environment");
        // Use a known pool on Ethereum (chain_id 1) e.g., PEPE/WETH
        let pool_address = "0xA43fe16908251ee70EF74718545e4FE6C5cCEc9f"; // PEPE/WETH 0.3% on Uniswap V3
        let chain_id = 1; // Ethereum
        let interval = "15m";
        let limit = Some(10);

        let result = fallback
            .fetch_candlesticks(
                pool_address,
                chain_id.to_string(),
                interval,
                limit,
            )
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
        let fallback = EvmFallback::from_env()
            .expect("Failed to create EvmFallback from environment");
        let address = "0x0000000000000000000000000000000000000000";
        let chain_id = 99999; // Unsupported chain ID

        let result_info = fallback
            .fetch_token_info(address, chain_id.to_string())
            .await;
        assert!(result_info.is_err());
        assert!(result_info
            .unwrap_err()
            .to_string()
            .contains("Unsupported chain ID"));

        let result_candles = fallback
            .fetch_candlesticks(
                address,
                chain_id.to_string(),
                "15m",
                Some(10),
            )
            .await;
        assert!(result_candles.is_err());
        assert!(result_candles
            .unwrap_err()
            .to_string()
            .contains("Unsupported chain ID"));
    }

    #[tokio::test]
    async fn test_unsupported_interval() {
        let fallback = EvmFallback::from_env()
            .expect("Failed to create EvmFallback from environment");
        let pool_address = "0xA43fe16908251ee70EF74718545e4FE6C5cCEc9f";
        let chain_id = 1;
        let interval = "1y"; // Unsupported interval

        let result_candles = fallback
            .fetch_candlesticks(
                pool_address,
                chain_id.to_string(),
                interval,
                Some(10),
            )
            .await;
        assert!(result_candles.is_err());
        assert!(result_candles
            .unwrap_err()
            .to_string()
            .contains("Unsupported interval format"));
    }

    #[tokio::test]
    async fn test_nonexistent_token() {
        let fallback = EvmFallback::from_env()
            .expect("Failed to create EvmFallback from environment");
        // Use a clearly invalid address
        let address = "0x000000000000000000000000000000000000dead";
        let chain_id = 1; // Ethereum

        let result = fallback
            .fetch_token_info(address, chain_id.to_string())
            .await;
        // Expecting a 404 or similar error from the API, mapped to anyhow::Error
        assert!(result.is_err());
        println!("Nonexistent token error: {:?}", result.unwrap_err());
    }

    #[tokio::test]
    async fn test_nonexistent_pool() {
        let fallback = EvmFallback::from_env()
            .expect("Failed to create EvmFallback from environment");
        let pool_address = "0x000000000000000000000000000000000000dead";
        let chain_id = 1;
        let interval = "15m";
        let limit = Some(10);

        let result = fallback
            .fetch_candlesticks(
                pool_address,
                chain_id.to_string(),
                interval,
                limit,
            )
            .await;
        // Expecting a 404 or similar error from the API
        assert!(result.is_err());
        println!("Nonexistent pool error: {:?}", result.unwrap_err());
    }
}
