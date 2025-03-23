use anyhow::{anyhow, Result};
use mcp_core::{tool_text_content, types::ToolResponseContent};
use mcp_core_macros::tool;

const API_BASE: &str = "https://api.listen-rs.com/v1/adapter";

#[tool(description = "
Fetch the latest price for a token from the Listen API.

Parameters:
- mint (string): The token's mint/pubkey address
")]
async fn fetch_price(mint: String) -> Result<ToolResponseContent> {
    let response = reqwest::get(format!("{}/price?mint={}", API_BASE, mint))
        .await
        .map_err(|e| anyhow!("Failed to fetch chart: {}", e))?;

    let data = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    if let Some(price) = data.get("price") {
        Ok(tool_text_content!(serde_json::to_string(&price)?))
    } else {
        Err(anyhow!("No price found for token: {}", mint))
    }
}

#[tool(description = "
Fetch top tokens from the Listen API.

No point using limit of more than ~6, less is more, as long as the filters are right

Lower timeframes work best, 7200 seconds is the sweet spot

Parameters:
- limit (string): Optional number of tokens to return
- min_volume (string): Optional minimum 24h volume filter
- min_market_cap (string): Minimum market cap filter
- max_market_cap (string): Maximum market cap filter, ~3B is the max to include all tokens
- timeframe (string): Optional timeframe in seconds, e.g. 86400 for the last 24 hours
- only_pumpfun_tokens (string): Optional boolean to filter only PumpFun tokens (default: \"true\")

Use the min_market_cap of 100k unless specified otherwise.

Returns a list of top tokens with their market data.
")]
pub async fn fetch_top_tokens(
    limit: String,
    min_volume: String,
    min_market_cap: String,
    max_market_cap: String,
    timeframe: String,
    only_pumpfun_tokens: String,
) -> Result<ToolResponseContent> {
    let mut url = format!("{}/top-tokens", API_BASE);
    let mut query_params = vec![];

    query_params.push(format!("limit={}", limit));
    query_params.push(format!("min_volume={}", min_volume));
    query_params.push(format!("min_market_cap={}", min_market_cap));
    query_params.push(format!("max_market_cap={}", max_market_cap));
    query_params.push(format!("timeframe={}", timeframe));
    query_params.push(format!("only_pumpfun_tokens={}", only_pumpfun_tokens));

    url = format!("{}?{}", url, query_params.join("&"));

    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("Failed to fetch top tokens: {}", e))?;

    let tokens = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(tool_text_content!(serde_json::to_string(&tokens)?))
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
async fn fetch_price_chart(mint: String, interval: String) -> Result<ToolResponseContent> {
    let response = reqwest::get(format!(
        "{}/candlesticks?mint={}&interval={}",
        API_BASE, mint, interval
    ))
    .await
    .map_err(|e| anyhow!("Failed to fetch chart: {}", e))?;

    let candlesticks = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(tool_text_content!(serde_json::to_string(&candlesticks)?))
}

#[tool(description = "
Fetch token metadata from the Listen API. This is the
metadata that was initially set during token creation by the token creator
that lives on-chain and IPFS.

Parameters:
- mint (string): The token's mint/pubkey address

It returns metadata that includes:
- Basic SPL token info (supply, decimals, authorities)
- MPL (Metaplex) metadata (name, symbol, URI) 
- IPFS metadata (name, description, image, social links)
")]
async fn fetch_token_metadata(mint: String) -> Result<ToolResponseContent> {
    let response = reqwest::get(format!("{}/metadata?mint={}", API_BASE, mint))
        .await
        .map_err(|e| anyhow!("Failed to fetch token metadata: {}", e))?;

    let metadata = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse JSON {}", e))?;

    Ok(tool_text_content!(serde_json::to_string(&metadata)?))
}
