use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;

use crate::data::TopToken;

pub const SOLANA_ONLY: bool = true;

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub timestamp: String,
    pub error_code: i32,
    pub error_message: Option<String>,
    pub elapsed: i32,
    pub credit_count: i32,
    pub notice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsdQuote {
    pub price: Option<f64>,
    pub volume_24h: Option<f64>,
    pub volume_change_24h: Option<f64>,
    pub percent_change_1h: Option<f64>,
    pub percent_change_24h: Option<f64>,
    pub percent_change_7d: Option<f64>,
    pub percent_change_30d: Option<f64>,
    pub percent_change_60d: Option<f64>,
    pub percent_change_90d: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_dominance: Option<f64>,
    pub fully_diluted_market_cap: Option<f64>,
    pub tvl: Option<f64>,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    #[serde(rename = "USD")]
    pub usd: UsdQuote,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coin {
    pub id: i64,
    pub name: String,
    pub symbol: String,
    pub slug: String,
    pub num_market_pairs: i32,
    pub date_added: String,
    pub tags: Vec<String>,
    pub max_supply: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub is_active: Option<i32>,
    pub infinite_supply: bool,
    pub platform: Option<serde_json::Value>,
    pub cmc_rank: Option<i32>,
    pub is_fiat: Option<i32>,
    pub self_reported_circulating_supply: Option<f64>,
    pub self_reported_market_cap: Option<f64>,
    pub tvl_ratio: Option<f64>,
    pub last_updated: String,
    pub quote: Quote,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryData {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub num_tokens: i32,
    pub last_updated: String,
    pub avg_price_change: f64,
    pub market_cap: f64,
    pub market_cap_change: f64,
    pub volume: f64,
    pub volume_change: f64,
    pub coins: Vec<Coin>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryResponse {
    pub status: Status,
    pub data: CategoryData,
}

impl CategoryResponse {
    pub fn to_top_tokens(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<TopToken>> {
        let limit = limit.unwrap_or(10);
        let mut tokens: Vec<TopToken> = self
            .data
            .coins
            .iter()
            .filter_map(|coin| {
                let usd_quote = &coin.quote.usd;

                // Skip if we don't have essential data
                let (price, volume_24h, market_cap) = match (
                    usd_quote.price,
                    usd_quote.volume_24h,
                    usd_quote.market_cap,
                ) {
                    (Some(p), Some(v), Some(m)) => (p, v, m),
                    _ => return None,
                };

                // Get chain_id from platform if available
                let chain_id = coin.platform.as_ref().and_then(|p| {
                    serde_json::from_value::<serde_json::Value>(p.clone())
                        .ok()
                        .and_then(|v| {
                            v.get("slug")
                                .and_then(|s| s.as_str())
                                .map(String::from)
                        })
                });

                // Get token address from platform if available
                let address = coin
                    .platform
                    .as_ref()
                    .and_then(|p| {
                        serde_json::from_value::<serde_json::Value>(p.clone())
                            .ok()
                            .and_then(|v| {
                                v.get("token_address")
                                    .and_then(|s| s.as_str())
                                    .map(String::from)
                            })
                    })
                    .unwrap_or_else(|| coin.symbol.clone());

                Some(TopToken {
                    name: coin.name.clone(),
                    pubkey: address,
                    price,
                    market_cap,
                    volume_24h,
                    price_change_24h: usd_quote
                        .percent_change_24h
                        .unwrap_or(0.0),
                    chain_id,
                    pools: vec![], // CMC doesn't provide pool information
                })
            })
            .collect();

        // Sort by volume
        tokens.sort_by(|a, b| {
            b.volume_24h
                .partial_cmp(&a.volume_24h)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if SOLANA_ONLY {
            return Ok(tokens
                .into_iter()
                .filter(|token| {
                    token.chain_id.clone().unwrap_or("".to_string())
                        == "solana".to_string()
                })
                .take(limit)
                .collect());
        }

        Ok(tokens.into_iter().take(limit).collect())
    }
}

pub async fn fetch_tokens_by_category(
    category_id: &str,
    limit: Option<usize>,
) -> Result<Vec<TopToken>> {
    let api_key = env::var("COINMARKETCAP_API_KEY").map_err(|_| {
        anyhow!("COINMARKETCAP_API_KEY environment variable not set")
    })?;

    let mut headers = HeaderMap::new();
    headers.insert("X-CMC_PRO_API_KEY", HeaderValue::from_str(&api_key)?);
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();
    let url = format!(
        "https://pro-api.coinmarketcap.com/v1/cryptocurrency/category?id={}",
        category_id
    );

    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .json::<CategoryResponse>()
        .await?;

    response.to_top_tokens(limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_deserialize_cmc_category_response() {
        let json = fs::read_to_string("mocks/cmc-tokens-by-category.json")
            .expect("Failed to read mock file");

        let response: CategoryResponse = serde_json::from_str(&json)
            .expect("Failed to deserialize mock response");

        assert!(!response.data.coins.is_empty());
    }

    #[test]
    fn test_convert_cmc_response_to_top_tokens() {
        let json = fs::read_to_string("mocks/cmc-tokens-by-category.json")
            .expect("Failed to read mock file");

        let response: CategoryResponse = serde_json::from_str(&json)
            .expect("Failed to deserialize mock response");

        let top_tokens = response
            .to_top_tokens(Some(10))
            .expect("Failed to convert to top tokens");

        assert!(!top_tokens.is_empty());
        assert!(top_tokens.len() <= 10);

        // Verify sorting by volume
        for i in 1..top_tokens.len() {
            assert!(
                top_tokens[i - 1].volume_24h >= top_tokens[i].volume_24h,
                "Tokens should be sorted by volume in descending order"
            );
        }

        // Print for inspection
        println!(
            "Top tokens: {}",
            serde_json::to_string_pretty(&top_tokens).unwrap()
        );
    }
}
