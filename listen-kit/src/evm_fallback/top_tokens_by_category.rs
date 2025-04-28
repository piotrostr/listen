use crate::data::TopToken;

use super::EvmFallback;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenRelationship {
    pub data: TokenData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenData {
    pub id: String,
    #[serde(rename = "type")]
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkRelationship {
    pub data: NetworkData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkData {
    pub id: String,
    #[serde(rename = "type")]
    pub network_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexRelationship {
    pub data: DexData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexData {
    pub id: String,
    #[serde(rename = "type")]
    pub dex_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceChangePercentage {
    pub m5: Option<String>,
    pub m15: Option<String>,
    pub m30: Option<String>,
    pub h1: Option<String>,
    pub h6: Option<String>,
    pub h24: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolAttributes {
    pub base_token_price_usd: String,
    pub quote_token_price_usd: String,
    pub address: String,
    pub name: String,
    pub pool_created_at: String,
    pub fdv_usd: Option<String>,
    pub market_cap_usd: Option<String>,
    pub price_change_percentage: PriceChangePercentage,
    pub reserve_in_usd: String,
    pub h24_volume_usd: String,
    pub h24_tx_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolRelationships {
    pub base_token: TokenRelationship,
    pub quote_token: TokenRelationship,
    pub network: NetworkRelationship,
    pub dex: DexRelationship,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pool {
    pub id: String,
    #[serde(rename = "type")]
    pub pool_type: String,
    pub attributes: PoolAttributes,
    pub relationships: PoolRelationships,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryPoolsResponse {
    pub data: Vec<Pool>,
}

impl EvmFallback {
    pub async fn fetch_pools_by_category(
        &self,
        category_id: &str,
        page: Option<u32>,
    ) -> Result<CategoryPoolsResponse> {
        let mut url =
            format!("{}/categories/{}/pools", self.base_url, category_id);

        // Add query parameters if provided
        let mut query_parts = Vec::new();

        if let Some(page) = page {
            query_parts.push(format!("page={}", page));
        }

        // Add sorting by 24h volume
        query_parts.push("sort=h24_volume_usd_desc".to_string());

        if !query_parts.is_empty() {
            url.push('?');
            url.push_str(&query_parts.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("x-cg-pro-api-key", self.api_key.clone())
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
                "GeckoTerminal API request failed for category pools ({}): {} - {}",
                url,
                status,
                error_text
            ));
        }

        let pools_response =
            response
                .json::<CategoryPoolsResponse>()
                .await
                .context("Failed to deserialize category pools response")?;

        Ok(pools_response)
    }

    pub fn top_tokens_from_category_pools_response(
        &self,
        response: CategoryPoolsResponse,
        limit: Option<usize>,
    ) -> Result<Vec<TopToken>> {
        let limit = limit.unwrap_or(10);

        // Create a map to aggregate data by token ID
        let mut token_data: std::collections::HashMap<
            String,
            (TopToken, f64),
        > = std::collections::HashMap::new();

        for pool in response.data {
            let base_token = &pool.relationships.base_token.data;
            let base_price: f64 =
                pool.attributes.base_token_price_usd.parse()?;
            let volume_24h: f64 = pool.attributes.h24_volume_usd.parse()?;

            // Skip if market cap is zero or missing
            let market_cap = match pool.attributes.market_cap_usd.as_ref() {
                Some(cap) => match cap.parse::<f64>() {
                    Ok(val) if val > 0.0 => val,
                    _ => continue,
                },
                None => continue,
            };

            // Parse 24h price change, default to 0.0 if not available
            let price_change_24h = pool
                .attributes
                .price_change_percentage
                .h24
                .as_ref()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            // Get chain ID based on network
            let chain_id = match pool.relationships.network.data.id.as_str() {
                "base" => Some(8453),
                "ethereum" => Some(1),
                "solana" => None,
                _ => None,
            };

            let entry = token_data
                .entry(base_token.id.clone())
                .or_insert_with(|| {
                    (
                        TopToken {
                            name: pool
                                .attributes
                                .name
                                .split(" / ")
                                .next()
                                .unwrap_or("")
                                .to_string(),
                            pubkey: base_token.id.clone(),
                            price: base_price,
                            market_cap,
                            volume_24h: 0.0, // Will accumulate
                            price_change_24h,
                            chain_id,
                        },
                        0.0,
                    ) // Track total volume for weighted average
                });

            // Accumulate volume and update data
            entry.0.volume_24h += volume_24h;
            entry.1 += volume_24h;
        }

        // Convert to vector, sort by volume, and limit results
        let mut tokens: Vec<TopToken> = token_data
            .into_iter()
            .map(|(_, (token, _))| token)
            .collect();

        tokens.sort_by(|a, b| {
            b.volume_24h
                .partial_cmp(&a.volume_24h)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(tokens.into_iter().take(limit).collect())
    }

    pub async fn fetch_top_tokens_by_category(
        &self,
        category_id: &str,
        page: Option<u32>,
        limit: Option<usize>,
    ) -> Result<Vec<TopToken>> {
        let pools_response =
            self.fetch_pools_by_category(category_id, page).await?;
        // debug
        println!(
            "pools_response: {}",
            serde_json::to_string_pretty(&pools_response).unwrap()
        );
        self.top_tokens_from_category_pools_response(pools_response, limit)
    }
}

// TODO there is no RWA here to put as an option, something that would be useful though
const _CATEGORIES: [&str; 5] = ["ai-agents", "animal", "cat", "dog", "ai"];

// sort these by volume

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_pools_by_category() {
        let client =
            EvmFallback::from_env().expect("Failed to create client");

        let result =
            client.fetch_pools_by_category("ai-agents", Some(1)).await;

        assert!(result.is_ok());
        let pools = result.unwrap();
        assert!(!pools.data.is_empty());

        // Test first pool data
        let first_pool = &pools.data[0];
        assert!(!first_pool.id.is_empty());
        assert!(!first_pool.attributes.address.is_empty());

        println!("pools: {}", serde_json::to_string_pretty(&pools).unwrap());
        // dump to json for debugging
        std::fs::write(
            "mocks/pools-by-category.json",
            serde_json::to_string_pretty(&pools).unwrap(),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_top_tokens_from_category_pools_response() {
        let client =
            EvmFallback::from_env().expect("Failed to create client");
        let response =
            std::fs::read_to_string("mocks/pools-by-category.json").unwrap();
        let pools: CategoryPoolsResponse =
            serde_json::from_str(&response).unwrap();

        // Test with default limit
        let top_tokens = client
            .top_tokens_from_category_pools_response(pools, None)
            .unwrap();

        assert!(top_tokens.len() <= 10);
    }

    #[tokio::test]
    async fn test_e2e_top_tokens_by_category() {
        let client =
            EvmFallback::from_env().expect("Failed to create client");
        let top_tokens = client
            .fetch_top_tokens_by_category("ai-agents", None, Some(10))
            .await
            .unwrap();
        println!(
            "top_tokens: {}",
            serde_json::to_string_pretty(&top_tokens).unwrap()
        );
    }
}
