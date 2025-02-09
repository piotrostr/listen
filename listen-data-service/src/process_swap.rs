use std::sync::Arc;

use crate::{
    constants::{USDC_MINT_KEY_STR, WSOL_MINT_KEY_STR},
    kv_store::RedisKVStore,
    message_queue::{MessageQueue, RedisMessageQueue},
    metadata::get_token_metadata,
    price::PriceUpdate,
    raydium_intruction_processor::Diff,
    sol_price_stream::SOL_PRICE_CACHE,
};
use anyhow::Result;
use chrono::Utc;
use tracing::info;

pub async fn calculate_price_for_wsol(token_amount: f64, sol_amount: f64) -> Result<f64> {
    let sol_price = SOL_PRICE_CACHE.get_price().await;
    let price = (sol_amount.abs() / token_amount.abs()) * sol_price;
    Ok(price)
}

pub async fn calculate_price_for_usdc(token_amount: f64, usdc_amount: f64) -> Result<f64> {
    Ok(usdc_amount.abs() / token_amount.abs())
}

pub async fn process_swap(
    diffs: Vec<Diff>,
    slot: u64,
    message_queue: &RedisMessageQueue,
    kv_store: &Arc<RedisKVStore>,
) -> Result<()> {
    // Only process swaps with exactly 2 tokens
    if diffs.len() != 2 {
        info!("Skipping swap with {} diffs", diffs.len());
        return Ok(());
    }

    // skip tiny swaps
    if diffs[0].diff.abs() < 0.0001 || diffs[1].diff.abs() < 0.0001 {
        info!("Skipping swap with tiny diffs");
        return Ok(());
    }

    let (token0, token1) = (&diffs[0], &diffs[1]);

    // Get absolute values of diffs since they're opposite signs
    let amount0 = token0.diff.abs();
    let amount1 = token1.diff.abs();

    // Determine which token is WSOL or USDC (if any)
    let (price, coin_mint) = match (token0.mint.as_str(), token1.mint.as_str()) {
        (WSOL_MINT_KEY_STR, other_mint) => {
            let price = calculate_price_for_wsol(amount1, amount0).await?;
            (price, other_mint)
        }
        (other_mint, WSOL_MINT_KEY_STR) => {
            let price = calculate_price_for_wsol(amount0, amount1).await?;
            (price, other_mint)
        }
        (USDC_MINT_KEY_STR, other_mint) => {
            let price = calculate_price_for_usdc(amount1, amount0).await?;
            (price, other_mint)
        }
        (other_mint, USDC_MINT_KEY_STR) => {
            let price = calculate_price_for_usdc(amount0, amount1).await?;
            (price, other_mint)
        }
        _ => return Ok(()), // Skip pairs without SOL or USDC
    };

    // Get metadata for the non-WSOL/USDC token
    let token_metadata = get_token_metadata(&kv_store, &coin_mint).await?;

    // Calculate market cap if we have the metadata
    let market_cap = token_metadata.as_ref().map(|metadata| {
        let supply = metadata.spl.supply as f64;
        let adjusted_supply = supply / (10_f64.powi(metadata.spl.decimals as i32));
        price * adjusted_supply
    });

    // Get token name from metadata, fallback to mint address
    let name = token_metadata
        .map(|m| m.mpl.name)
        .unwrap_or_else(|| coin_mint.to_string());

    // Create and publish price update
    let price_update = PriceUpdate {
        name,
        pubkey: coin_mint.to_string(),
        price,
        market_cap,
        timestamp: Utc::now().timestamp(),
        slot,
    };

    info!("price_update: {:#?}", price_update);
    // info!(
    //     "jupiter price: {}",
    //     crate::util::get_jup_price(coin_mint.to_string())
    //         .await
    //         .unwrap()
    // );

    message_queue.publish_price_update(price_update).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::util::round_to_decimals;

    use super::*;

    #[tokio::test]
    async fn test_sol_for_token() {
        let diffs = vec![
            Diff {
                mint: "G6ZaVuWEuGtFRooaiHQWjDzoCzr2f7BWr3PhsQRnjSTE".to_string(),
                pre_amount: 9502698.632123,
                post_amount: 9493791.483438,
                diff: -8907.148685000837,
                owner: "8CNuwDVRshWyZtWRvgb31AMaBge4q6KSRHNPdJHP29HU".to_string(),
            },
            Diff {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                pre_amount: 145.774357667,
                post_amount: 142.421949398,
                diff: -3.3524082689999943,
                owner: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string(),
            },
        ];

        // hard-set the price to what it was at the time of the swap
        SOL_PRICE_CACHE.set_price(201.36).await;

        let price = calculate_price_for_wsol(diffs[0].diff, diffs[1].diff)
            .await
            .unwrap();
        let rounded_price = round_to_decimals(price, 4);
        assert!(rounded_price == 0.0758, "price: {}", rounded_price);
    }

    #[tokio::test]
    async fn test_sol_for_token_2() {
        let diffs = vec![
            Diff {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                pre_amount: 450.295597127,
                post_amount: 450.345597127,
                diff: 0.05000000000001137,
                owner: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string(),
            },
            Diff {
                mint: "CSChJMDH1drnxaN5ZXr8ZPZtqXv2FJqNTGcSujyfmoon".to_string(),
                pre_amount: 61602947.9232689,
                post_amount: 61596125.50088912,
                diff: -6822.422379776835,
                owner: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string(),
            },
        ];

        // hard-set the price to what it was at the time of the swap
        SOL_PRICE_CACHE.set_price(202.12).await;

        let price = calculate_price_for_wsol(diffs[1].diff, diffs[0].diff)
            .await
            .unwrap();
        let rounded_price = round_to_decimals(price, 5);
        assert!(rounded_price == 0.00148, "price: {}", rounded_price);
    }
}
