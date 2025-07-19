use anyhow::{anyhow, Result};

pub mod evm_fallback_tools;
pub mod listen_api_tools;
pub mod stocks;
pub mod twitter_tools;
pub mod unified;
pub mod web_tools;

pub use listen_api_tools::*;
pub use stocks::*;
pub use twitter_tools::*;
pub use unified::*;
pub use web_tools::*;

pub fn candlesticks_and_analysis_to_price_action_analysis_response(
    candlesticks: Vec<Candlestick>,
    analysis: String,
) -> Result<PriceActionAnalysisResponse> {
    let mut sorted_candlesticks = candlesticks.clone();
    sorted_candlesticks.sort_by_key(|c| c.timestamp);

    let latest_candle = sorted_candlesticks
        .last()
        .ok_or_else(|| anyhow!("No candlesticks available"))?;
    let first_candle = sorted_candlesticks
        .first()
        .ok_or_else(|| anyhow!("No candlesticks available"))?;

    let total_volume: f64 =
        sorted_candlesticks.iter().map(|c| c.volume).sum();
    let high = sorted_candlesticks
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = sorted_candlesticks
        .iter()
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min);
    let price_change = ((latest_candle.close - first_candle.open)
        / first_candle.open)
        * 100.0;

    Ok(PriceActionAnalysisResponse {
        analysis,
        current_price: latest_candle.close,
        current_time: chrono::Utc::now().to_rfc3339(),
        total_volume,
        price_change,
        high,
        low,
    })
}
