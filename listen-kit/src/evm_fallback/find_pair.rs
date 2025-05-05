use super::{map_chain_id_to_network, EvmFallback};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct VolumeUsd {
    h24: String,
}

#[derive(Debug, Deserialize)]
struct PoolAttributes {
    address: String,
    volume_usd: VolumeUsd,
}

#[derive(Debug, Deserialize)]
struct Pool {
    attributes: PoolAttributes,
}

#[derive(Debug, Deserialize)]
struct PoolResponse {
    data: Vec<Pool>,
}

impl EvmFallback {
    pub async fn find_pair_address(
        &self,
        token_address: &str,
        chain_id: u64,
    ) -> Result<Option<String>> {
        let network = map_chain_id_to_network(chain_id)?;

        log::debug!(
            "Resolving pair address for {} on {}",
            token_address,
            network
        );

        let url = format!(
            "{}/networks/{}/tokens/{}/pools",
            self.base_url, network, token_address
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
                "GeckoTerminal API request failed ({}): {} - {}",
                url,
                status,
                error_text
            ));
        }

        let pool_response = response
            .json::<PoolResponse>()
            .await
            .context("Failed to deserialize pools response")?;

        // Sort pools by 24h volume
        let mut pools = pool_response.data;
        pools.sort_by(|a, b| {
            let vol_a =
                a.attributes.volume_usd.h24.parse::<f64>().unwrap_or(0.0);
            let vol_b =
                b.attributes.volume_usd.h24.parse::<f64>().unwrap_or(0.0);
            vol_b.partial_cmp(&vol_a).unwrap()
        });

        // Get the highest volume pool address
        Ok(pools.first().map(|pool| pool.attributes.address.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_pair_address() {
        let client =
            EvmFallback::from_env().expect("Failed to create client");
        // PEPE token on Ethereum
        let result = client
            .find_pair_address(
                "0x6982508145454ce325ddbe47a25d4ec3d2311933",
                1,
            )
            .await;

        assert!(result.is_ok());
        let pair = result.unwrap();
        assert!(pair.is_some());
        println!("Found pair address: {:?}", pair);
    }
}
