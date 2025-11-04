use super::{map_chain_id_to_network, EvmFallback};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtTokenMetadata {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: Option<u32>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub websites: Option<Vec<String>>,
    pub chain_id: String,
    pub discord_url: Option<String>,
    pub telegram_handle: Option<String>,
    pub twitter_handle: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GTTokenInfoAttributes {
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
}

#[derive(Deserialize, Debug)]
pub struct GTTokenInfoData {
    attributes: GTTokenInfoAttributes,
}

#[derive(Deserialize, Debug)]
pub struct GTTokenInfoResponse {
    data: GTTokenInfoData,
}

impl EvmFallback {
    pub async fn fetch_token_info(
        &self,
        address: &str,
        chain_id: String,
    ) -> Result<GtTokenMetadata> {
        let network = map_chain_id_to_network(chain_id.clone())?;
        let url = format!(
            "{}/networks/{}/tokens/{}/info",
            self.base_url, network, address
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
}
