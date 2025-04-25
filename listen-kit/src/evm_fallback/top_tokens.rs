use crate::data::TopToken;

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

        let mut trending_pools_resp =
            response.json::<GTTrendingPoolsResponse>().await.context(
                "Failed to deserialize GeckoTerminal trending pools response",
            )?;

        let mut top_tokens = Vec::new();

        // filter out the ones with 0 mc
        trending_pools_resp.data = trending_pools_resp
            .data
            .into_iter()
            .filter(|pool| {
                let market_cap = pool
                    .attributes
                    .market_cap_usd
                    .clone()
                    .unwrap_or("0".to_string());
                market_cap != "0"
            })
            .collect();

        for pool in trending_pools_resp.data.iter().take(limit) {
            // Extract token name from pool name (assuming format is "TOKEN / OTHER")
            let token_name = pool
                .attributes
                .name
                .split(" / ")
                .next()
                .unwrap_or("")
                .to_string();

            // Extract token address from base_token id
            // Format is usually "network_address"
            let token_id = &pool.relationships.base_token.data.id;
            let token_address =
                token_id.split('_').nth(1).unwrap_or(token_id).to_string();

            // Parse numeric values with fallbacks to 0.0 for missing data
            let price: f64 =
                pool.attributes.base_token_price_usd.parse().unwrap_or(0.0);

            let market_cap: f64 = match &pool.attributes.market_cap_usd {
                Some(cap) => cap.parse().unwrap_or(0.0),
                None => 0.0,
            };

            let volume_24h: f64 = match &pool.attributes.volume_usd.h24 {
                Some(vol) => vol.parse().unwrap_or(0.0),
                None => 0.0,
            };

            let price_change_24h: f64 =
                match &pool.attributes.price_change_percentage.h24 {
                    Some(change) => change.parse().unwrap_or(0.0),
                    None => 0.0,
                };

            top_tokens.push(TopToken {
                name: token_name,
                pubkey: token_address,
                price,
                market_cap,
                volume_24h,
                price_change_24h,
                chain_id: Some(chain_id),
            });
        }

        Ok(top_tokens)
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
