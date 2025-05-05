use anyhow::Result;
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
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceInfo {
    pub latest_price: f64,
    pub ema_price_ticks: Option<Vec<SimplePriceTick>>,
    pub price_ticks_timeframe: Option<String>,
    pub total_volume: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub metadata: serde_json::Value,
    pub price_info: PriceInfo,
}

#[tool(description = "
Get the token info - the metadata, recent price, socials and more.

Parameters:
- address (string): The address of the token to fetch metadata for
")]
pub async fn get_token(address: String, chain_id: u64) -> Result<Token> {
    if address.starts_with("0x") {
        get_token_evm(address, chain_id).await
    } else {
        get_token_solana(address).await
    }
}

async fn get_token_evm(address: String, chain_id: u64) -> Result<Token> {
    let evm_fallback = EvmFallback::from_env()?;
    let (metadata, candlesticks) = tokio::try_join!(
        evm_fallback.fetch_token_info(&address, chain_id),
        evm_fallback.fetch_candlesticks(&address, chain_id, "15m", Some(200))
    )?;

    let price_info = candlesticks_and_timeframe_to_price_info(
        candlesticks,
        "15m".to_string(),
    );

    Ok(Token {
        metadata: serde_json::to_value(metadata)?,
        price_info,
    })
}

async fn get_token_solana(address: String) -> Result<Token> {
    validate_mint(&address)?;

    let (metadata, candlesticks) = tokio::try_join!(
        fetch_token_metadata(address.clone()),
        fetch_candlesticks(address, "15m".to_string())
    )?;

    let price_info = candlesticks_and_timeframe_to_price_info(
        candlesticks,
        "15m".to_string(),
    );

    Ok(Token {
        metadata,
        price_info,
    })
}

pub fn candlesticks_and_timeframe_to_price_info(
    mut candlesticks: Vec<Candlestick>,
    timeframe: String,
) -> PriceInfo {
    // Sort by timestamp ascending (oldest first)
    candlesticks.sort_by_key(|c| c.timestamp);

    let period = 10.0;
    let multiplier = 2.0 / (period + 1.0);

    let mut ema_ticks = Vec::with_capacity(candlesticks.len());
    let mut current_ema = candlesticks[0].close;
    let total_volume: f64 = candlesticks.iter().map(|c| c.volume).sum();

    for stick in candlesticks.iter() {
        current_ema =
            stick.close * multiplier + current_ema * (1.0 - multiplier);

        ema_ticks.push(SimplePriceTick {
            price: current_ema,
            timestamp: stick.timestamp,
        });
    }

    PriceInfo {
        latest_price: current_ema,
        ema_price_ticks: Some(ema_ticks),
        price_ticks_timeframe: Some(timeframe),
        total_volume: Some(total_volume),
    }
}
