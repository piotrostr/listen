use anyhow::Result;
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::dexscreener::{search_ticker, PairInfo};
use crate::solana::balance::Holding;

pub async fn fetch_pair_info(mint_or_symbol: String) -> Result<PairInfo> {
    let res = search_ticker(mint_or_symbol.clone()).await?;

    let mut matching_pairs: Vec<&PairInfo> = res
        .pairs
        .iter()
        .filter(|pair| pair.chain_id == "solana")
        .collect();

    matching_pairs.sort_by(|a, b| {
        b.volume
            .h24
            .partial_cmp(&a.volume.h24)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // get the pair with the highest liquidity
    matching_pairs
        .first()
        .map(|pair| (*pair).clone()) // Dereference and clone the PairInfo
        .ok_or_else(|| {
            anyhow::anyhow!("No matching pairs found for {}", mint_or_symbol)
        })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMetadata {
    address: String,
    name: String,
    symbol: String,
    decimals: u8,
    #[serde(rename = "logoURI")]
    logo_uri: String,
    #[serde(rename = "daily_volume", default)]
    volume_24h: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceData {
    id: String,
    #[serde(rename = "type")]
    price_type: String,
    price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    data: std::collections::HashMap<String, Option<PriceData>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioItem {
    address: String,
    name: String,
    symbol: String,
    decimals: u8,
    #[serde(rename = "logoURI")]
    logo_uri: String,
    price: f64,
    amount: f64,
    daily_volume: f64,
}

pub async fn holdings_to_portfolio(
    holdings: Vec<Holding>,
) -> Result<Vec<PortfolioItem>> {
    let client = Client::new();

    // Fetch metadata for all tokens
    let metadata_futures: Vec<_> = holdings
        .iter()
        .map(|holding| {
            let client = &client;
            async move {
                let url =
                    format!("https://tokens.jup.ag/token/{}", holding.mint);
                client.get(&url).send().await?.json::<TokenMetadata>().await
            }
        })
        .collect();

    let metadata_results = join_all(metadata_futures).await;
    let token_metadata: Vec<TokenMetadata> = metadata_results
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    // Fetch prices for all tokens
    let mints: Vec<_> = holdings.iter().map(|h| h.mint.as_str()).collect();
    let prices_url =
        format!("https://api.jup.ag/price/v2?ids={}", mints.join(","));
    let price_response: PriceResponse =
        client.get(&prices_url).send().await?.json().await?;

    // Combine all data into portfolio items
    let portfolio: Vec<PortfolioItem> = holdings
        .iter()
        .zip(token_metadata.iter())
        .map(|(holding, metadata)| {
            let price = price_response
                .data
                .get(&holding.mint)
                .map(|p| match p {
                    Some(price_data) => {
                        price_data.price.parse::<f64>().unwrap_or(0.0)
                    }
                    None => 0.0,
                })
                .unwrap_or(0.0);

            let amount = holding.amount as f64
                / (10f64.powi(metadata.decimals as i32));

            PortfolioItem {
                address: metadata.address.clone(),
                name: metadata.name.clone(),
                symbol: metadata.symbol.clone(),
                decimals: metadata.decimals,
                logo_uri: metadata.logo_uri.clone(),
                price,
                amount,
                daily_volume: metadata.volume_24h.unwrap_or(0.0),
            }
        })
        .collect();

    Ok(portfolio)
}

#[cfg(test)]
mod tests {
    use solana_sdk::signer::Signer;

    use super::*;
    use crate::solana::balance::get_holdings;
    use crate::solana::util::{load_keypair_for_tests, make_rpc_client};

    #[tokio::test]
    async fn test_holdings_to_portfolio() {
        let holdings = get_holdings(
            &make_rpc_client(),
            &load_keypair_for_tests().pubkey(),
        )
        .await
        .unwrap();

        holdings_to_portfolio(holdings).await.unwrap();
    }
}
