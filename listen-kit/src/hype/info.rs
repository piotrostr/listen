use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, CandlesSnapshotResponse, InfoClient};
use rig_tool_macro::tool;

use crate::{
    common::spawn_with_signer_and_channel,
    data::{
        candlesticks_and_analysis_to_price_action_analysis_response,
        Candlestick, PriceActionAnalysisResponse,
    },
    distiller::analyst::{humanize_timestamp, Analyst},
    hype::parse_evm_address,
    reasoning_loop::ReasoningLoop,
    signer::SignerContext,
};

#[tool(description = "
Gets the complete orderbook snapshot for a given coin. Example response:
{
  \"coin\": \"ETH\",
  \"levels\": [
    [
      {\"n\": 1, \"px\": \"2545.4\", \"sz\": \"11.7811\"}, // 1 order at 2545.4, size 11.7811 ETH
      {\"n\": 12, \"px\": \"2545.0\", \"sz\": \"136.8789\"}, // 12 orders at 2545.0, size 136.8789 ETH
      {\"n\": 17, \"px\": \"2544.9\", \"sz\": \"144.4251\"}, // 17 orders at 2544.9, size 144.4251 ETH
      // ... more orders deeper on the bid (buy) side, skipped for brevity
    ],
    [
      {\"n\": 1, \"px\": \"2545.5\", \"sz\": \"0.0061\"}, // 1 order at 2545.5, size 0.0061 ETH
      {\"n\": 10, \"px\": \"2545.6\", \"sz\": \"40.0728\"}, // 10 orders at 2545.6, size 40.0728 ETH
      {\"n\": 6, \"px\": \"2545.7\", \"sz\": \"102.1028\"}, // 6 orders at 2545.7, size 102.1028 ETH
      // ... more orders deeper on the ask (sell) side, skipped for brevity
    ]
  ],
  \"time\": 1748279333332
}
")]
pub async fn get_l2_snapshot(coin: String) -> Result<serde_json::Value> {
    // thread-local this, possibly onto the signer
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let info = client.l2_snapshot(coin).await?;
    Ok(serde_json::to_value(info)?)
}

#[tool(
    description = "Gets the open orders on the Hyperliquid exchange for the current user"
)]
pub async fn get_open_orders() -> Result<serde_json::Value> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let address = SignerContext::current().await.address();
    let res = client.open_orders(parse_evm_address(address)?).await?;
    Ok(serde_json::to_value(res)?)
}

#[tool(
    description = "Gets the Hyperliquid balance overview of the current user. Response involves a summary of all of the asset positions as well as the margin summary (account value, total margin used)."
)]
pub async fn get_balance_overview() -> Result<serde_json::Value> {
    let address = SignerContext::current().await.address();
    let parsed_address = parse_evm_address(address)?;

    let (spot, perp) = tokio::join!(
        _get_balance_overview_spot(parsed_address.clone()),
        _get_balance_overview_perp(parsed_address)
    );

    let spot = spot?;
    let perp = perp?;

    // Combine the results into a single JSON value
    Ok(serde_json::json!({
        "spotBalances": spot,
        "perpBalances": perp
    }))
}

#[tool(description = "Gets the latest price for a coin. Example response:
{
  \"bid\": 2545.4,
  \"ask\": 2545.5
}
")]
pub async fn get_latest_price(coin: String) -> Result<serde_json::Value> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let res = client.l2_snapshot(coin).await?;
    let bid = res.levels[0][0].px.parse::<f64>()?;
    let ask = res.levels[1][0].px.parse::<f64>()?;
    Ok(serde_json::json!({
        "bid": bid,
        "ask": ask,
    }))
}

pub async fn _get_balance_overview_perp(
    address: H160,
) -> Result<serde_json::Value> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let res = client.user_state(address).await?;
    Ok(serde_json::to_value(res)?)
}

pub async fn _get_balance_overview_spot(
    address: H160,
) -> Result<serde_json::Value> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let res = client.user_token_balances(address).await?;
    Ok(serde_json::to_value(res)?)
}

#[tool(description = "
Gets the raw candlestick data for a coin. In the OHLCV format, the response is a
list of candles, each with the following fields:
- t: timestamp
- o: open
- h: high
- l: low
- c: close
- v: volume

Example:
{{
  \"coin\": \"ETH\",
  \"interval\": \"1m\",
  \"limit\": \"100\"
}}

This method can be useful for fetching a small chunk of price action, say
the last 5 5m candles. For larger timeframes, to save context and extract
the most relevant information, use the get_candlesticks_analysis tool.
")]
pub async fn get_candlesticks_raw(
    coin: String,
    interval: String,
    limit: String,
) -> Result<serde_json::Value> {
    let res = get_candles_snapshot(coin, &interval, limit).await?;
    leanify_candles_snapshot_response(res)
}

pub async fn get_candles_snapshot(
    coin: String,
    interval: &str,
    limit: String,
) -> Result<Vec<CandlesSnapshotResponse>> {
    let limit = limit.parse::<u64>()?;
    if limit < 1 || limit > 200 {
        // limit over 200 yields too much
        return Err(anyhow::anyhow!("Limit must be between 1 and 200"));
    }
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let (start, end) = calculate_start_and_end_times(interval, limit)?;
    client
        .candles_snapshot(coin, interval.to_string(), start, end)
        .await
        .map_err(|e| anyhow!("Failed to get candles snapshot: {}", e))
}

#[tool(description = "
Gets the price action analysis for a coin.

Parameters:
- coin: The coin to get the analysis for
- interval: The interval to get the analysis for
- limit: The limit of candles to get the analysis for
- intent: The intent of the analysis

Example:
{{
  \"coin\": \"ETH\",
  \"interval\": \"1m\",
  \"limit\": \"100\",
  \"intent\": \"What is the price action for ETH in the last 100 1m candles?\"
}}

Returns the price action analysis for the coin from the Chart Analyst agent
along with summary of the price action - the current price, time, total volume,
price change in the period analysed and the high/low price.

This is the go-to method for getting a high-level overview of the price action, while condensing
the verbose candlesticks output
")]
pub async fn get_candlesticks_analysis(
    coin: String,
    interval: String,
    limit: String,
    intent: Option<String>,
) -> Result<PriceActionAnalysisResponse> {
    let candles =
        get_candles_snapshot(coin, &interval, limit.clone()).await?;
    let ctx = SignerContext::current().await;
    let locale = ctx.locale();
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|e| anyhow!("Failed to create Analyst: {}", e))?;
    let channel = ReasoningLoop::get_current_stream_channel().await;
    let _candlesticks = candles_snapshot_to_candlesticks(&candles)?;
    let candlesticks = _candlesticks.clone();
    let analysis =
        spawn_with_signer_and_channel(ctx, channel, move || async move {
            analyst
                .analyze_chart(&_candlesticks, &interval, intent)
                .await
                .map_err(|e| anyhow!("Failed to analyze chart: {}", e))
        })
        .await
        .await??;

    candlesticks_and_analysis_to_price_action_analysis_response(
        candlesticks,
        analysis,
    )
}

pub fn candles_snapshot_to_candlesticks(
    res: &Vec<CandlesSnapshotResponse>,
) -> Result<Vec<Candlestick>> {
    let mut data = Vec::new();
    for candle in res {
        let candlestick = Candlestick {
            timestamp: candle.time_open / 1000,
            close: candle.close.parse::<f64>()?,
            high: candle.high.parse::<f64>()?,
            low: candle.low.parse::<f64>()?,
            open: candle.open.parse::<f64>()?,
            volume: candle.vlm.parse::<f64>()?,
        };
        data.push(candlestick);
    }
    Ok(data)
}

pub fn leanify_candles_snapshot_response(
    res: Vec<CandlesSnapshotResponse>,
) -> Result<serde_json::Value> {
    let mut data = Vec::new();
    for candle in res {
        data.push(serde_json::json!({
            "t": humanize_timestamp(candle.time_open / 1000)?,
            "o": candle.open,
            "h": candle.high,
            "l": candle.low,
            "c": candle.close,
            "v": candle.vlm,
        }));
    }
    Ok(serde_json::to_value(data)?)
}

pub const ALLOWED_INTERVALS: [&str; 11] = [
    "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d",
];

pub async fn get_symbol_list() -> Result<Vec<String>> {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;
    let res = client.all_mids().await?;
    Ok(res.keys().cloned().collect())
}

fn calculate_start_and_end_times(
    interval: &str,
    limit: u64,
) -> Result<(u64, u64)> {
    if limit > 5000 {
        return Err(anyhow::anyhow!("Limit must be less than 5000"));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("System time error: {}", e))?
        .as_millis() as u64;

    let interval_ms = match interval {
        "1m" => 60 * 1000,
        "3m" => 3 * 60 * 1000,
        "5m" => 5 * 60 * 1000,
        "15m" => 15 * 60 * 1000,
        "30m" => 30 * 60 * 1000,
        "1h" => 60 * 60 * 1000,
        "2h" => 2 * 60 * 60 * 1000,
        "4h" => 4 * 60 * 60 * 1000,
        "8h" => 8 * 60 * 60 * 1000,
        "12h" => 12 * 60 * 60 * 1000,
        "1d" => 24 * 60 * 60 * 1000,
        "3d" => 3 * 24 * 60 * 60 * 1000,
        "1w" => 7 * 24 * 60 * 60 * 1000,
        "1M" => 30 * 24 * 60 * 60 * 1000,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid interval: {}, must be one of {:?}",
                interval,
                ALLOWED_INTERVALS
            ))
        }
    };

    let start = now - (interval_ms * limit);
    Ok((start, now))
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use ethers::signers::LocalWallet;

    use super::*;

    const TEST_ADDRESS: &str = "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770";

    #[tokio::test]
    async fn test_get_balance_overview_perp() {
        let res = _get_balance_overview_perp(TEST_ADDRESS.parse().unwrap())
            .await
            .unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }

    #[tokio::test]
    async fn test_get_balance_overview_spot() {
        let res = _get_balance_overview_spot(TEST_ADDRESS.parse().unwrap())
            .await
            .unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }

    #[tokio::test]
    async fn test_get_candlesticks_raw() {
        let res = get_candlesticks_raw(
            "ETH".to_string(),
            "1m".to_string(),
            "100".to_string(),
        )
        .await
        .unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }

    #[tokio::test]
    async fn test_get_symbol_list() {
        get_symbol_list().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_candlesticks_analysis() {
        let private_key = std::env::var("ETHEREUM_PRIVATE_KEY").unwrap();
        let signer: LocalWallet = private_key.parse().unwrap();
        SignerContext::with_signer(Arc::new(signer), async {
            let res = get_candlesticks_analysis(
                "ETH".to_string(),
                "1m".to_string(),
                "100".to_string(),
                None,
            )
            .await
            .unwrap();
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
            Ok(())
        })
        .await
        .unwrap();
    }
}
