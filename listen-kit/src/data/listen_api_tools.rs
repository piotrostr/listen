use crate::data::candlesticks_and_analysis_to_price_action_analysis_response;
use crate::distiller::analyst::Analyst;
use crate::reasoning_loop::ReasoningLoop;
use crate::signer::SignerContext;
use crate::{
    common::spawn_with_signer_and_channel, solana::util::validate_mint,
};
use anyhow::{anyhow, Result};
use rig_tool_macro::tool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Candlestick {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceTick {
    pub timestamp: u64,
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceChart {
    pub price_ticks: Vec<PriceTick>,
    pub pct_change: f64,
    pub interval: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolInfo {
    pub dex: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopToken {
    pub name: String,
    pub pubkey: String,
    pub price: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub chain_id: Option<String>,
    #[serde(default)]
    pub pools: Vec<PoolInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceActionAnalysisResponse {
    pub analysis: String,
    pub current_price: f64,
    pub current_time: String,
    pub total_volume: f64,
    pub price_change: f64,
    pub high: f64,
    pub low: f64,
}

pub const LISTEN_API_BASE: &str = "https://api.listen-rs.com/v1/adapter";

#[tool(description = "
Fetch token metadata for any Solana token from the Listen API. This is the metadata that was
initially set during token creation by the token creator that lives on-chain and
IPFS.

Parameters:
- mint (string): The token's mint/pubkey address

It returns metadata that includes:
- Basic SPL token info (supply, decimals, authorities)
- MPL (Metaplex) metadata (name, symbol, URI) 
- IPFS metadata (name, description, image, social links)
")]
pub async fn fetch_token_metadata(mint: String) -> Result<serde_json::Value> {
    validate_mint(&mint)?;

    let response =
        reqwest::get(format!("{}/metadata?mint={}", LISTEN_API_BASE, mint))
            .await
            .map_err(|e| anyhow!("Failed to fetch token metadata: {}", e))?;

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse JSON {}", e))
}

#[tool(description = "
Fetch token price for any Solana token from the Listen API.

Parameters:
- mint (string): The token's mint/pubkey address

Returns the price of the token in USD.
")]
pub async fn fetch_token_price(mint: String) -> Result<f64> {
    validate_mint(&mint)?;

    let response =
        reqwest::get(format!("{}/price?mint={}", LISTEN_API_BASE, mint))
            .await
            .map_err(|e| anyhow!("Failed to fetch token price: {}", e))?;

    let data = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    let price = match data["price"].as_f64() {
        Some(price) => price,
        None => return Err(anyhow!("Failed to parse price: {}", data)),
    };

    Ok(price)
}

#[tool(description = "
Fetch top Solana tokens based on volume from the Listen API.

Parameters:
- limit (optional, string): number of tokens to return (default: 4)
- min_market_cap (optional, string): minimum market cap filter (default: 1000000)
- max_market_cap (optional, string): maximum market cap filter (default: no limit)
- timeframe (optional, string): timeframe in seconds (default: 7200)

Keep the default set of params unless the user asks for something different

Returns a list of top tokens with their market data.
")]
pub async fn fetch_top_tokens(
    limit: Option<String>,
    min_market_cap: Option<String>,
    max_market_cap: Option<String>,
    timeframe: Option<String>,
) -> Result<Vec<TopToken>> {
    let limit = limit.unwrap_or("4".to_string());
    let min_market_cap = min_market_cap.unwrap_or("1000000".to_string());
    let max_market_cap = max_market_cap.unwrap_or("0".to_string());
    let timeframe = timeframe.unwrap_or("7200".to_string());

    let mut url = format!("{}/top-tokens", LISTEN_API_BASE);
    let mut query_params = vec![];

    query_params.push(format!("limit={}", limit));
    query_params.push(format!("min_market_cap={}", min_market_cap));
    if max_market_cap != "0" {
        query_params.push(format!("max_market_cap={}", max_market_cap));
    }
    query_params.push(format!("timeframe={}", timeframe));
    query_params.push("only_pumpfun_tokens=true".to_string());

    if !query_params.is_empty() {
        url = format!("{}?{}", url, query_params.join("&"));
    }

    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("Failed to fetch top tokens: {}", e))?;

    let tokens = response
        .json::<Vec<TopToken>>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(tokens)
}

#[tool(description = "
Fetch price series for any Solana token from the Listen API.

Parameters:
- mint (string): The token's mint/pubkey address
- interval (string): The interval of the price data, one of:
  * '5m'  (5 minutes)
  * '15m' (15 minutes)
  * '30m' (30 minutes)
  * '1h'  (1 hour)
  * '4h'  (4 hours)
  * '1d'  (1 day)
")]
pub async fn fetch_price_chart(
    mint: String,
    interval: String,
) -> Result<Vec<PriceTick>> {
    validate_mint(&mint)?;

    let candlesticks = fetch_candlesticks(mint, interval).await?;

    let price_ticks = candlesticks
        .iter()
        .map(|candlestick| PriceTick {
            timestamp: candlestick.timestamp,
            price: candlestick.close,
            volume: candlestick.volume,
        })
        .collect::<Vec<PriceTick>>();

    Ok(price_ticks)
}

#[tool(description = "
Fetch price action analysis based on candlestick data for any Solana token from the Listen API.

Parameters:
- mint (string): The token's mint/pubkey address
- interval (string): The candlestick interval, one of:
  * '15m' (15 minutes)
  * '30m' (30 minutes)
  * '1h'  (1 hour)
  * '4h'  (4 hours)
  * '1d'  (1 day)
- intent (string, optional): The intent of the analysis, passed on to the Chart Analyst agent

start with 1m interval, 200 limit and work up the timeframes if required

Returns an analysis of the chart from the Chart Analyst agent
")]
pub async fn fetch_price_action_analysis(
    mint: String,
    interval: String,
    intent: Option<String>,
) -> Result<PriceActionAnalysisResponse> {
    validate_mint(&mint)?;

    // Validate interval
    match interval.as_str() {
        "15s" | "30s" | "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "1d" => {}
        _ => return Err(anyhow!("Invalid interval: {}", interval)),
    }

    if mint.starts_with("0x") {
        return Err(anyhow!(
            "Invalid mint: {}, use fetch_price_action_analysis_evm instead for EVM tokens",
            mint
        ));
    }

    let candlesticks = fetch_candlesticks(mint, interval.clone()).await?;

    let candlesticks_clone = candlesticks.clone();

    let ctx = SignerContext::current().await;
    let locale = ctx.locale();
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|e| anyhow!("Failed to create Analyst: {}", e))?;

    let channel = ReasoningLoop::get_current_stream_channel().await;

    let analysis =
        spawn_with_signer_and_channel(ctx, channel, move || async move {
            analyst
                .analyze_chart(&candlesticks, &interval, intent)
                .await
                .map_err(|e| anyhow!("Failed to analyze chart: {}", e))
        })
        .await
        .await??;

    candlesticks_and_analysis_to_price_action_analysis_response(
        candlesticks_clone,
        analysis,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_top_tokens() {
        fetch_top_tokens(
            Some("10".to_string()),
            Some("1000000000000000000".to_string()),
            Some("1000000000000000000".to_string()),
            Some("1d".to_string()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_fetch_price_action_analysis() {
        // FIXME thread local signer needs init
        let analysis = fetch_price_action_analysis(
            "61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump".to_string(),
            "5m".to_string(),
            None,
        )
        .await;
        tracing::info!("{:?}", analysis);
    }
}

pub async fn fetch_candlesticks(
    mint: String,
    interval: String,
) -> Result<Vec<Candlestick>> {
    let url = format!(
        "{}/candlesticks?mint={}&interval={}",
        LISTEN_API_BASE, mint, interval
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("Failed to fetch candlesticks: {}", e))?;

    response
        .json::<Vec<Candlestick>>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))
}
