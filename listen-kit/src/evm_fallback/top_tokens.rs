use crate::data::{PoolInfo, TopToken};

use super::{map_chain_id_to_network, EvmFallback};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GTTokenId {
    id: String,
    #[serde(rename = "type")]
    _token_type: String,
}

#[derive(Deserialize, Debug)]
pub struct GTTokenRelationship {
    data: GTTokenId,
}

#[derive(Deserialize, Debug)]
pub struct GTPriceChangePercentage {
    h24: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GTVolumeUsd {
    h24: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GTPoolAttributes {
    name: String,
    address: String,
    base_token_price_usd: String,
    market_cap_usd: Option<String>,
    price_change_percentage: GTPriceChangePercentage,
    volume_usd: GTVolumeUsd,
}

#[derive(Deserialize, Debug)]
pub struct GTPoolRelationships {
    base_token: GTTokenRelationship,
}

#[derive(Deserialize, Debug)]
pub struct GTPoolData {
    #[serde(rename = "id")]
    _id: String,
    attributes: GTPoolAttributes,
    relationships: GTPoolRelationships,
}

#[derive(Deserialize, Debug)]
pub struct GTTrendingPoolsResponse {
    data: Vec<GTPoolData>,
}

const DURATION_OPTIONS: &[&str] = &["5m", "1h", "6h", "24h"];

impl EvmFallback {
    pub async fn fetch_top_tokens(
        &self,
        chain_id: u64,
        duration: String,
        limit: usize,
    ) -> Result<Vec<TopToken>> {
        let network = map_chain_id_to_network(chain_id)?;
        if !DURATION_OPTIONS.contains(&duration.as_str()) {
            return Err(anyhow!(
                "Invalid duration: {}, must be one of: {}",
                duration,
                DURATION_OPTIONS.join(", ")
            ));
        }

        let url = format!(
            "{}/networks/{}/trending_pools?duration={}&page=1",
            self.base_url, network, duration
        );

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
                "GeckoTerminal API request failed for trending pools ({}): {} - {}",
                url,
                status,
                error_text
            ));
        }

        let trending_pools_resp =
            response.json::<GTTrendingPoolsResponse>().await.context(
                "Failed to deserialize GeckoTerminal trending pools response",
            )?;

        // Create a map to aggregate data by token address
        let mut token_data: std::collections::HashMap<
            String,
            (TopToken, f64),
        > = std::collections::HashMap::new();

        // Filter and process pools
        for pool in trending_pools_resp.data.iter() {
            // Skip if market cap is zero or missing
            let market_cap = match &pool.attributes.market_cap_usd {
                Some(cap) => match cap.parse::<f64>() {
                    Ok(val) if val > 0.0 => val,
                    _ => continue,
                },
                None => continue,
            };

            // Extract token address from base_token id
            let token_id = &pool.relationships.base_token.data.id;
            let token_address =
                token_id.split('_').nth(1).unwrap_or(token_id).to_string();

            let price: f64 =
                pool.attributes.base_token_price_usd.parse().unwrap_or(0.0);
            let volume_24h: f64 = pool
                .attributes
                .volume_usd
                .h24
                .as_ref()
                .and_then(|vol| vol.parse().ok())
                .unwrap_or(0.0);
            let price_change_24h: f64 = pool
                .attributes
                .price_change_percentage
                .h24
                .as_ref()
                .and_then(|change| change.parse().ok())
                .unwrap_or(0.0);

            // Create pool info
            let pool_info = PoolInfo {
                dex: "".to_string(), // No dex info available in this endpoint
                address: pool.attributes.address.clone(),
            };

            let entry = token_data
                .entry(token_address.clone())
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
                            pubkey: token_address.clone(),
                            price,
                            market_cap,
                            volume_24h: 0.0,
                            price_change_24h,
                            chain_id: Some(chain_id.to_string()),
                            pools: Vec::new(),
                        },
                        0.0,
                    )
                });

            // Accumulate volume and add pool info
            entry.0.volume_24h += volume_24h;
            entry.1 += volume_24h;
            entry.0.pools.push(pool_info);
        }

        // Convert to vector, sort by volume, and limit results
        let mut top_tokens: Vec<TopToken> = token_data
            .into_iter()
            .map(|(_, (token, _))| token)
            .collect();

        top_tokens.sort_by(|a, b| {
            b.volume_24h
                .partial_cmp(&a.volume_24h)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(top_tokens.into_iter().take(limit).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_top_tokens() {
        let evm_fallback = EvmFallback::from_env()
            .expect("Failed to create EvmFallback from environment");
        let top_tokens = evm_fallback
            .fetch_top_tokens(1, "24h".to_string(), 10)
            .await
            .expect("Failed to fetch top tokens");
        println!("Top tokens: {:?}", top_tokens);
    }
}
