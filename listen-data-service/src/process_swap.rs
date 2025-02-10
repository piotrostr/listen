use std::sync::Arc;

use crate::constants::WSOL_MINT_KEY_STR;
use crate::diffs::{get_token_balance_diff, process_diffs, Diff, DiffsResult};
use crate::{
    db::{ClickhouseDb, Database},
    kv_store::RedisKVStore,
    message_queue::{MessageQueue, RedisMessageQueue},
    metadata::get_token_metadata,
    price::PriceUpdate,
    sol_price_stream::SOL_PRICE_CACHE,
};
use anyhow::Result;
use carbon_core::transaction::TransactionMetadata;
use chrono::Utc;
use tracing::{debug, info, warn};

pub async fn process_swap(
    transaction_metadata: &TransactionMetadata,
    message_queue: &RedisMessageQueue,
    kv_store: &Arc<RedisKVStore>,
    db: &Arc<ClickhouseDb>,
) -> Result<()> {
    let diffs = get_token_balance_diff(
        transaction_metadata
            .meta
            .pre_token_balances
            .as_ref()
            .unwrap(),
        transaction_metadata
            .meta
            .post_token_balances
            .as_ref()
            .unwrap(),
    );

    // skip tiny swaps
    if diffs.iter().all(|d| d.diff.abs() < 0.01) {
        debug!("skipping tiny diffs");
        return Ok(());
    }

    let sol_price = SOL_PRICE_CACHE.get_price().await;

    if diffs.len() > 3 {
        warn!(
            "https://solscan.io/tx/{} Skipping swap with unexpected number of tokens {:#?}",
            transaction_metadata.signature, diffs
        );
        return Ok(());
    }

    // Handle multi-hop swaps (3 tokens)
    if diffs.len() == 3 {
        // Find the tokens with positive and negative changes
        let mut positive_diff = None;
        let mut negative_diff = None;
        let mut sol_diff = None;

        for diff in &diffs {
            if diff.mint == WSOL_MINT_KEY_STR {
                sol_diff = Some(diff);
                continue;
            }
            if diff.diff > 0.0 {
                positive_diff = Some(diff);
            } else if diff.diff < 0.0 {
                negative_diff = Some(diff);
            }
        }

        if positive_diff.is_none()
            || negative_diff.is_none()
            || sol_diff.is_none()
        {
            warn!(
                "https://solscan.io/tx/{} Skipping multi-hop swap with unexpected token changes {:#?}",
                transaction_metadata.signature, diffs
            );
            return Ok(());
        }

        if let (Some(pos), Some(neg), Some(sol)) =
            (positive_diff, negative_diff, sol_diff)
        {
            // Process first hop: token being sold to SOL
            process_two_token_swap(
                &vec![neg.clone(), sol.clone()],
                transaction_metadata,
                message_queue,
                kv_store,
                db,
                sol_price,
                true,
            )
            .await?;

            // Process second hop: SOL to token being bought
            process_two_token_swap(
                &vec![pos.clone(), sol.clone()],
                transaction_metadata,
                message_queue,
                kv_store,
                db,
                sol_price,
                true,
            )
            .await?;

            return Ok(());
        }
    }

    // Handle regular 2-token swaps
    if diffs.len() != 2 {
        warn!(
            "https://solscan.io/tx/{} Skipping swap with unexpected number of tokens {:#?}",
            transaction_metadata.signature, diffs
        );
        return Ok(());
    }

    process_two_token_swap(
        &diffs,
        transaction_metadata,
        message_queue,
        kv_store,
        db,
        sol_price,
        false,
    )
    .await
}

// Helper function to process a single two-token swap
async fn process_two_token_swap(
    diffs: &Vec<Diff>,
    transaction_metadata: &TransactionMetadata,
    message_queue: &RedisMessageQueue,
    kv_store: &Arc<RedisKVStore>,
    db: &Arc<ClickhouseDb>,
    sol_price: f64,
    multi_hop: bool,
) -> Result<()> {
    let DiffsResult {
        price,
        swap_amount,
        coin_mint,
        is_buy,
    } = match process_diffs(diffs, sol_price) {
        Ok(result) => result,
        Err(e) => {
            let token_mints =
                diffs.iter().map(|d| d.mint.clone()).collect::<Vec<_>>();
            warn!(?e, ?token_mints);
            return Ok(());
        }
    };

    // Get metadata and emit price update
    let token_metadata = get_token_metadata(kv_store, &coin_mint).await?;

    // Calculate market cap if we have the metadata
    let market_cap = token_metadata.as_ref().map(|metadata| {
        let supply = metadata.spl.supply as f64;
        let adjusted_supply =
            supply / (10_f64.powi(metadata.spl.decimals as i32));
        price * adjusted_supply
    });

    // Get token name from metadata, fallback to mint address
    let name = token_metadata
        .map(|m| m.mpl.name)
        .unwrap_or_else(|| coin_mint.to_string());

    let market_cap = market_cap.unwrap_or(0.0);

    let price_update = PriceUpdate {
        name,
        pubkey: coin_mint,
        price,
        market_cap,
        timestamp: Utc::now().timestamp(),
        slot: transaction_metadata.slot,
        swap_amount,
        owner: transaction_metadata.fee_payer.to_string(),
        signature: format!(
            "https://solscan.io/tx/{}",
            transaction_metadata.signature
        ),
        multi_hop,
        is_buy,
    };

    info!("price_update: {:#?}", price_update);

    db.insert_price(&price_update).await?;
    message_queue.publish_price_update(price_update).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        diffs::Diff,
        util::{make_rpc_client, round_to_decimals},
    };

    use super::*;

    #[tokio::test]
    async fn test_sol_for_token() {
        let diffs = vec![
            Diff {
                mint: "G6ZaVuWEuGtFRooaiHQWjDzoCzr2f7BWr3PhsQRnjSTE"
                    .to_string(),
                pre_amount: 9502698.632123,
                post_amount: 9493791.483438,
                diff: -8907.148685000837,
                owner: "8CNuwDVRshWyZtWRvgb31AMaBge4q6KSRHNPdJHP29HU"
                    .to_string(),
            },
            Diff {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                pre_amount: 145.774357667,
                post_amount: 142.421949398,
                diff: -3.3524082689999943,
                owner: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
                    .to_string(),
            },
        ];

        let DiffsResult {
            price, swap_amount, ..
        } = process_diffs(&diffs, 201.36).unwrap();
        let rounded_price = round_to_decimals(price, 4);
        assert!(rounded_price == 0.0758, "price: {}", rounded_price);
        assert!(
            swap_amount == 3.3524082689999943 * 201.36,
            "swap_amount: {}",
            swap_amount
        );
    }

    #[tokio::test]
    async fn test_sol_for_token_2() {
        let diffs = vec![
            Diff {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                pre_amount: 450.295597127,
                post_amount: 450.345597127,
                diff: 0.05000000000001137,
                owner: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
                    .to_string(),
            },
            Diff {
                mint: "CSChJMDH1drnxaN5ZXr8ZPZtqXv2FJqNTGcSujyfmoon"
                    .to_string(),
                pre_amount: 61602947.9232689,
                post_amount: 61596125.50088912,
                diff: -6822.422379776835,
                owner: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
                    .to_string(),
            },
        ];

        let DiffsResult {
            price, swap_amount, ..
        } = process_diffs(&diffs, 202.12).unwrap();
        let rounded_price = round_to_decimals(price, 5);
        assert!(rounded_price == 0.00148, "price: {}", rounded_price);
        assert!(
            swap_amount == 0.05000000000001137 * 202.12,
            "swap_amount: {}",
            swap_amount
        );
    }

    #[tokio::test]
    async fn test_by_signature() {
        let signature = "538voMuFQKp3oE6Tu598R8kJN12sum2cGMxZBxrV2Vuip1TL4qdWaXiJ8u3yRxgJy9SFX4faP2zC83oDX68D2wuW";
        let transaction = make_rpc_client()
            .unwrap()
            .get_transaction_with_config(
                &signature.parse().unwrap(),
                solana_client::rpc_config::RpcTransactionConfig {
                    encoding: Some(solana_transaction_status::UiTransactionEncoding::JsonParsed),
                    max_supported_transaction_version: Some(0),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let transaction_meta = transaction.transaction.meta.unwrap();

        let diffs = get_token_balance_diff(
            transaction_meta.pre_token_balances.as_ref().unwrap(),
            transaction_meta.post_token_balances.as_ref().unwrap(),
        );
        println!("diffs: {:#?}", diffs);
        let DiffsResult {
            price, swap_amount, ..
        } = process_diffs(&diffs, 203.67).unwrap();
        let rounded_price = round_to_decimals(price, 5);
        assert!(rounded_price == 0.00035, "price: {}", rounded_price);
        let rounded_swap_amount = round_to_decimals(swap_amount, 4);
        assert!(
            rounded_swap_amount == 0.8618,
            "swap_amount: {}",
            rounded_swap_amount
        );
    }
}
