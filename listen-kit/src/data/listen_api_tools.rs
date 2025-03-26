use crate::common::wrap_unsafe;
use crate::distiller::analyst::Analyst;
use crate::signer::SignerContext;
use anyhow::{anyhow, Result};
use rig_tool_macro::tool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
pub struct TopToken {
    pub name: String,
    pub pubkey: String,
    pub price: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
}

const API_BASE: &str = "https://api.listen-rs.com/v1/adapter";

#[tool(description = "
Fetch token metadata from the Listen API. This is the metadata that was
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
    let response =
        reqwest::get(format!("{}/metadata?mint={}", API_BASE, mint))
            .await
            .map_err(|e| anyhow!("Failed to fetch token metadata: {}", e))?;

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse JSON {}", e))
}

#[tool(description = "
Fetch top tokens from the Listen API.

No point using limit of more than ~6, less is more, as long as the filters are right

Lower timeframes work best, 7200 seconds is the sweet spot, you can request any
timeframe though, up to 24 hours

Parameters:
- limit (string): number of tokens to return
- min_market_cap (string): minimum market cap filter
- max_market_cap (string): maximum market cap filter
- timeframe (string): timeframe in seconds

Use the min_market_cap of 100k unless specified otherwise.
For max market cap, pass \"0\" for any market cap unless specified otherwise

Returns a list of top tokens with their market data.
")]
pub async fn fetch_top_tokens(
    limit: String,
    min_market_cap: String,
    max_market_cap: String,
    timeframe: String,
) -> Result<Vec<TopToken>> {
    let mut url = format!("{}/top-tokens", API_BASE);
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
Fetch price series for a token from the Listen API.

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
    let response = reqwest::get(format!(
        "{}/candlesticks?mint={}&interval={}",
        API_BASE, mint, interval
    ))
    .await
    .map_err(|e| anyhow!("Failed to fetch chart: {}", e))?;

    let candlesticks = response
        .json::<Vec<Candlestick>>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

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
Fetch price action analysis based on candlestick data for a token from the Listen API.

Parameters:
- mint (string): The token's mint/pubkey address
- interval (string): The candlestick interval, one of:
  * '15m' (15 minutes)
  * '30m' (30 minutes)
  * '1h'  (1 hour)
  * '4h'  (4 hours)
  * '1d'  (1 day)
- intent (string): The intent of the analysis, passed on to the Chart Analyst agent, possible to pass \"\" for no intent

start with 1m interval, 200 limit and work up the timeframes if required

Returns an analysis of the chart from the Chart Analyst agent
")]
pub async fn fetch_price_action_analysis(
    mint: String,
    interval: String,
    intent: String,
) -> Result<String> {
    // Validate interval
    match interval.as_str() {
        "15s" | "30s" | "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "1d" => {}
        _ => return Err(anyhow!("Invalid interval: {}", interval)),
    }

    let url = format!(
        "{}/candlesticks?mint={}&interval={}",
        API_BASE, mint, interval
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("Failed to fetch candlesticks: {}", e))?;

    let candlesticks = response
        .json::<Vec<Candlestick>>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    let locale = SignerContext::current().await.locale();
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|e| anyhow!("Failed to create Analyst: {}", e))?;

    wrap_unsafe(move || async move {
        analyst
            .analyze_chart(&candlesticks, &interval, Some(intent))
            .await
            .map_err(|e| anyhow!("Failed to analyze chart: {}", e))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_top_tokens() {
        fetch_top_tokens(
            "10".to_string(),
            "1000000000000000000".to_string(),
            "1000000000000000000".to_string(),
            "1d".to_string(),
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
            "".to_string(),
        )
        .await;
        println!("{:?}", analysis);
    }
}
