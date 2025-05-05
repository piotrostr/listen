use anyhow::{anyhow, Result};
use rig_tool_macro::tool;
use serde::{Deserialize, Serialize};

use crate::{
    data::{fetch_candlesticks, fetch_token_metadata, Candlestick},
    evm_fallback::EvmFallback,
    solana::util::validate_mint,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimplePriceTick {
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceInfo {
    pub latest_price: f64,
    pub ema_price_ticks: Vec<SimplePriceTick>,
    pub price_ticks_timeframe: String,
    pub total_volume: f64,
    pub pct_change: f64,
    pub period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub metadata: Option<serde_json::Value>,
    pub price_info: Option<PriceInfo>,
}

#[tool(description = "
Get the token info - the metadata, recent price, socials and more.

Parameters:
- address (string): The address of the token to fetch metadata for
- chain_id (u64): The chain ID of the token to fetch metadata for. Leave blank for Solana tokens.
")]
pub async fn get_token(
    address: String,
    chain_id: Option<u64>,
) -> Result<Token> {
    if let Some(chain_id) = chain_id {
        get_token_evm(address, chain_id).await
    } else {
        get_token_solana(address).await
    }
}

async fn get_token_evm(address: String, chain_id: u64) -> Result<Token> {
    let evm_fallback = EvmFallback::from_env()?;
    let pool_address =
        evm_fallback.find_pair_address(&address, chain_id).await?;
    if pool_address.is_none() {
        return Err(anyhow!("No pool address found for token"));
    }

    let pool_address = pool_address.unwrap();

    let (metadata_result, candlesticks_result) = tokio::join!(
        evm_fallback.fetch_token_info(&address, chain_id),
        evm_fallback.fetch_candlesticks(
            &pool_address,
            chain_id,
            "15m",
            Some(200)
        )
    );

    let metadata = metadata_result.ok();
    let price_info = match candlesticks_result {
        Ok(candlesticks) => candlesticks_and_timeframe_to_price_info(
            candlesticks,
            "15m".to_string(),
        )
        .ok(),
        Err(_) => None,
    };

    Ok(Token {
        metadata: serde_json::to_value(metadata).ok(),
        price_info,
    })
}

async fn get_token_solana(address: String) -> Result<Token> {
    validate_mint(&address)?;

    let (metadata_result, candlesticks_result) = tokio::join!(
        fetch_token_metadata(address.clone()),
        fetch_candlesticks(address, "15m".to_string())
    );

    let metadata = metadata_result.ok();
    let price_info = match candlesticks_result {
        Ok(candlesticks) => candlesticks_and_timeframe_to_price_info(
            candlesticks,
            "15m".to_string(),
        )
        .ok(),
        Err(_) => None,
    };

    Ok(Token {
        metadata,
        price_info,
    })
}

pub fn candlesticks_and_timeframe_to_price_info(
    mut candlesticks: Vec<Candlestick>,
    timeframe: String,
) -> Result<PriceInfo> {
    if candlesticks.is_empty() {
        return Err(anyhow!("No candlesticks data available"));
    }

    candlesticks.sort_by_key(|c| c.timestamp);

    // Increase EMA period for smoother line
    let period = 30.0;
    let multiplier = 2.0 / (period + 1.0);

    let first = candlesticks.first().expect("Already checked for empty");
    let last = candlesticks.last().expect("Already checked for empty");

    let mut ema_price_ticks = Vec::new();
    let mut current_ema = first.close;
    let total_volume: f64 = candlesticks.iter().map(|c| c.volume).sum();

    // Take fewer points for smoother visualization
    let sample_rate = 4; // adjust this to control smoothness
    for (i, stick) in candlesticks.iter().enumerate() {
        current_ema =
            stick.close * multiplier + current_ema * (1.0 - multiplier);

        if i % sample_rate == 0 {
            ema_price_ticks.push(SimplePriceTick { price: current_ema });
        }
    }

    // Ensure we always include the last point
    if !ema_price_ticks.is_empty()
        && ema_price_ticks.last().unwrap().price != current_ema
    {
        ema_price_ticks.push(SimplePriceTick { price: current_ema });
    }

    let pct_change = ((last.close - first.close) / first.close) * 100.0;
    let duration_secs = last.timestamp - first.timestamp;
    let duration_hours = duration_secs as f64 / 3600.0;
    let period = format!("last {:.1} hours", duration_hours);

    Ok(PriceInfo {
        latest_price: current_ema,
        ema_price_ticks,
        price_ticks_timeframe: timeframe,
        total_volume,
        pct_change,
        period,
    })
}
