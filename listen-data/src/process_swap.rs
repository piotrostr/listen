use crate::diffs::{
    extra_mint_details_from_tx_metadata, process_token_transfers, DiffsError,
    DiffsResult, TokenTransferDetails, SPL_TOKEN_TRANSFER_PROCESSOR,
};
use crate::{
    db::{ClickhouseDb, Database},
    kv_store::RedisKVStore,
    message_queue::{MessageQueue, RedisMessageQueue},
    metadata::get_token_metadata,
    metrics::SwapMetrics,
    price::PriceUpdate,
    sol_price_stream::get_sol_price,
};
use anyhow::{Context, Result};
use carbon_core::instruction::NestedInstruction;
use carbon_core::transaction::TransactionMetadata;
use chrono::Utc;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, warn};

static DEBUG: once_cell::sync::Lazy<bool> =
    once_cell::sync::Lazy::new(|| std::env::var("DEBUG").is_ok());

/// Validates whether a token transfer involves a known vault account.
///
/// This function checks if either the source or destination address of a token transfer
/// matches any address in the provided set of vault addresses. It's used to filter token
/// transfers that are relevant to DEX or AMM operations by ensuring they interact with
/// a liquidity pool vault.
/// For a real-world example, see:
/// https://solscan.io/tx/2usSAGxq35GJxQxVKHQ7NHBDnJim95Jyk3AeFrRAcpHc2TJUH3bjhVSvtAWcxnqnQyJFzpPFgJvMHNkTuQ8t779f
pub fn is_valid_vault_transfer(
    transfer: &TokenTransferDetails,
    vaults: &HashSet<String>,
    fee_adas: Option<&HashSet<String>>,
) -> bool {
    // Early return if it's a fee transfer
    if let Some(fee_adas) = fee_adas {
        if fee_adas.contains(&transfer.destination) {
            return false;
        }
    }
    vaults.contains(&transfer.destination) || vaults.contains(&transfer.source)
}

pub async fn process_swap(
    vaults: &HashSet<String>,
    fee_adas: Option<&HashSet<String>>,
    transaction_metadata: &TransactionMetadata,
    nested_instructions: &[NestedInstruction],
    message_queue: &RedisMessageQueue,
    kv_store: &Arc<RedisKVStore>,
    db: &Arc<ClickhouseDb>,
    metrics: &SwapMetrics,
) -> Result<()> {
    // Decrement pending swaps when this function exits
    let _pending_guard = PendingSwapGuard(metrics);

    let mint_details =
        extra_mint_details_from_tx_metadata(transaction_metadata);

    let inner_transfers = SPL_TOKEN_TRANSFER_PROCESSOR
        .decode_token_transfer_with_vaults_from_nested_instructions(
            nested_instructions,
            &mint_details,
        );
    let transfers = inner_transfers
        .into_iter()
        .filter(|d| is_valid_vault_transfer(d, vaults, fee_adas))
        .collect::<Vec<_>>();

    if transfers.iter().all(|d| d.ui_amount < 0.1) {
        debug!("skipping tiny diffs");
        metrics.increment_skipped_tiny_swaps();
        return Ok(());
    }

    if transfers.iter().any(|d| d.ui_amount == 0.0) {
        debug!("skipping zero diffs (arbitrage likely)");
        metrics.increment_skipped_zero_swaps();
        return Ok(());
    }

    let sol_price = get_sol_price().await;

    if transfers.len() > 3 || transfers.len() < 2 {
        debug!(
            "https://solscan.io/tx/{} skipping swap with unexpected number of tokens: {}",
            transaction_metadata.signature, transfers.len()
        );
        metrics.increment_skipped_unexpected_number_of_tokens();
        return Ok(());
    }

    process_two_token_swap(
        vaults,
        &transfers,
        transaction_metadata,
        message_queue,
        kv_store,
        db,
        metrics,
        sol_price,
        false,
    )
    .await
    .context("failed to process two token swap")
}

// Helper function to process a single two-token swap
#[allow(clippy::too_many_arguments)]
async fn process_two_token_swap(
    vaults: &HashSet<String>,
    transfers: &[TokenTransferDetails],
    transaction_metadata: &TransactionMetadata,
    message_queue: &RedisMessageQueue,
    kv_store: &Arc<RedisKVStore>,
    db: &Arc<ClickhouseDb>,
    metrics: &SwapMetrics,
    sol_price: f64,
    multi_hop: bool,
) -> Result<()> {
    let DiffsResult {
        price,
        swap_amount,
        coin_mint,
        is_buy,
    } = match process_token_transfers(vaults, transfers, sol_price) {
        Ok(result) => result,
        Err(e) => {
            match e {
                DiffsError::NonWsolsSwap => {
                    metrics.increment_skipped_non_wsol();
                }
                DiffsError::ExpectedExactlyTwoTokenBalanceDiffs => {
                    metrics.increment_skipped_unexpected_number_of_tokens();
                }
            }
            return Ok(());
        }
    };

    // Get metadata and emit price update
    let token_metadata = match get_token_metadata(kv_store, &coin_mint).await {
        Ok(Some(metadata)) => metadata,
        Ok(None) => {
            debug!(
                "https://solscan.io/tx/{} failed to get token metadata",
                transaction_metadata.signature
            );
            metrics.increment_skipped_no_metadata();
            return Ok(());
        }
        Err(e) => {
            warn!(
                "https://solscan.io/tx/{} failed to get token metadata: {}",
                transaction_metadata.signature, e
            );
            metrics.increment_skipped_no_metadata();
            return Ok(());
        }
    };

    // Calculate market cap if we have the metadata
    let market_cap = {
        let supply = token_metadata.spl.supply as f64;
        let adjusted_supply =
            supply / (10_f64.powi(token_metadata.spl.decimals as i32));
        price * adjusted_supply
    };

    let is_pump = token_metadata
        .mpl
        .ipfs_metadata
        .as_ref()
        .and_then(|metadata| metadata.get("createdOn"))
        .is_some_and(|value| {
            value.as_str().is_some_and(|s| s.contains("pump.fun"))
        });

    let price_update = PriceUpdate {
        name: token_metadata.mpl.name,
        pubkey: coin_mint,
        price,
        market_cap,
        timestamp: Utc::now().timestamp() as u64,
        slot: transaction_metadata.slot,
        swap_amount,
        owner: transaction_metadata.fee_payer.to_string(),
        signature: transaction_metadata.signature.to_string(),
        multi_hop,
        is_buy,
        is_pump,
    };

    metrics.set_latest_update_slot(transaction_metadata.slot);

    if *DEBUG {
        println!(
            "https://solscan.io/tx/{} {}: {} - {}",
            transaction_metadata.signature,
            transaction_metadata.slot,
            price_update.name,
            price_update.price,
        );
    }

    // Run all three database operations in parallel
    let db_future = db.insert_price(&price_update);
    let mq_future = message_queue.publish_price_update(price_update.clone());
    let kv_future = kv_store.insert_price(&price_update);

    let (db_result, mq_result, kv_result) =
        tokio::join!(db_future, mq_future, kv_future);

    // Handle results
    match db_result {
        Ok(_) => metrics.increment_db_insert_success(),
        Err(e) => {
            metrics.increment_db_insert_failure();
            return Err(e);
        }
    }

    match mq_result {
        Ok(_) => metrics.increment_message_send_success(),
        Err(e) => {
            metrics.increment_message_send_failure();
            return Err(e.into());
        }
    }

    match kv_result {
        Ok(_) => metrics.increment_kv_insert_success(),
        Err(e) => {
            metrics.increment_kv_insert_failure();
            return Err(e);
        }
    }

    Ok(())
}

// Helper struct to decrement pending swaps when dropped
struct PendingSwapGuard<'a>(&'a SwapMetrics);

impl Drop for PendingSwapGuard<'_> {
    fn drop(&mut self) {
        self.0.decrement_pending_swaps();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{make_rpc_client, round_to_decimals};
    use carbon_core::{
        datasource::TransactionUpdate,
        instruction::NestedInstructions,
        transformers::{
            extract_instructions_with_metadata,
            transaction_metadata_from_original_meta,
        },
    };
    use solana_sdk::signature::Signature;
    use std::str::FromStr;
    use tracing::error;

    // are all examples of transactions where both raydium and whirlpool or meteora are used simultaneously
    // https://solscan.io/tx/31pB39KowUTdDSjXhzCYi7QxVSWSM4ZijaSWAkCduWUUR6GuGrWwVBbcXLLdJnVLrWbQaV7YFL2SigBXRatGfnji
    // IQ-SOL
    #[tokio::test]
    async fn test_token_for_sol() {
        let mut vaults = HashSet::new();
        vaults
            .insert("HqDtzxBsHHhmTHbzmUk5aJkAZE8iGf6KKeeYrh4mVCc3".to_string());
        vaults
            .insert("6M2KAV658rer6g2L7tAAQtXK7f1GmrbG7ycW14gHdK5U".to_string());
        let diffs = vec![
            TokenTransferDetails {
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                mint: "AsyfR3e5JcPqWot4H5MMhQUm7DZ4zwQrcp2zbB7vpump"
                    .to_string(),
                source: "3oV3EFEp6GUTt8cn3swj1oQXhmeuRyKv9cEzpSVZga5K"
                    .to_string(),
                destination: "HqDtzxBsHHhmTHbzmUk5aJkAZE8iGf6KKeeYrh4mVCc3"
                    .to_string(),
                authority: "6LXutJvKUw8Q5ue2gCgKHQdAN4suWW8awzFVC6XCguFx"
                    .to_string(),
                decimals: 6,
                amount: 279274681533,
                ui_amount: 279274.681533,
            },
            TokenTransferDetails {
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "6M2KAV658rer6g2L7tAAQtXK7f1GmrbG7ycW14gHdK5U"
                    .to_string(),
                destination: "BuqEDKUwyAotZuK37V4JYEykZVKY8qo1zKbpfU9gkJMo"
                    .to_string(),
                authority: "BuqEDKUwyAotZuK37V4JYEykZVKY8qo1zKbpfU9gkJMo"
                    .to_string(),
                decimals: 9,
                amount: 856978344,
                ui_amount: 8.56978344,
            },
        ];

        let DiffsResult {
            is_buy,
            price,
            swap_amount,
            ..
        } = process_token_transfers(&vaults, &diffs, 201.36).unwrap();
        let rounded_price = round_to_decimals(price, 4);
        assert!(!is_buy, "is_buy: {}", is_buy);
        assert!(rounded_price == 0.0062, "price: {}", rounded_price);
        assert!(
            swap_amount == 8.56978344 * 201.36,
            "swap_amount: {}",
            swap_amount
        );
    }

    // https://solscan.io/tx/31pB39KowUTdDSjXhzCYi7QxVSWSM4ZijaSWAkCduWUUR6GuGrWwVBbcXLLdJnVLrWbQaV7YFL2SigBXRatGfnji
    // SOL-Fullsend
    #[tokio::test]
    async fn test_sol_for_token() {
        let mut vaults = HashSet::new();
        vaults
            .insert("Ej7C1F58YLJRLHS5eyovmUeFyX5Xc8999ZZxrgYABPZi".to_string());
        vaults
            .insert("84gHbaT9Eq4SF4uQ5cR2zaaP13coaHyrTnnUY7hSVaYL".to_string());
        let diffs = vec![
            TokenTransferDetails {
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "BuqEDKUwyAotZuK37V4JYEykZVKY8qo1zKbpfU9gkJMo"
                    .to_string(),
                destination: "Ej7C1F58YLJRLHS5eyovmUeFyX5Xc8999ZZxrgYABPZi"
                    .to_string(),
                authority: "6LXutJvKUw8Q5ue2gCgKHQdAN4suWW8awzFVC6XCguFx"
                    .to_string(),
                decimals: 9,
                amount: 856832000,
                ui_amount: 0.856832,
            },
            TokenTransferDetails {
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                mint: "AshG5mHt4y4etsjhKFb2wA2rq1XZxKks1EPzcuXwpump"
                    .to_string(),
                source: "84gHbaT9Eq4SF4uQ5cR2zaaP13coaHyrTnnUY7hSVaYL"
                    .to_string(),
                destination: "C4XmPzBYkdsEmq6CXgL8TZxfniqWBxu5ft1gRhiUMvia"
                    .to_string(),
                authority: "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
                    .to_string(),
                decimals: 6,
                amount: 2469387663,
                ui_amount: 2469.387663,
            },
        ];

        let DiffsResult {
            price,
            swap_amount,
            is_buy,
            ..
        } = process_token_transfers(&vaults, &diffs, 201.36).unwrap();
        let rounded_price = round_to_decimals(price, 5);
        assert!(rounded_price == 0.06987, "price: {}", rounded_price);
        assert!(
            swap_amount == 0.856832 * 201.36,
            "swap_amount: {}",
            swap_amount
        );
        assert!(is_buy, "is_buy: {}", is_buy);
    }

    async fn get_transaction(
        signature: &str,
        outer_index: usize,
        inner_index: Option<usize>,
    ) -> Result<Vec<TokenTransferDetails>> {
        let signature =
            Signature::from_str(signature).expect("failed to make signature");
        let encoded_transaction = make_rpc_client()
            .unwrap()
            .get_transaction_with_config(
                &signature,
                solana_client::rpc_config::RpcTransactionConfig {
                    encoding: Some(solana_transaction_status::UiTransactionEncoding::Binary),
                    max_supported_transaction_version: Some(0),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let transaction = encoded_transaction.transaction;
        let meta_original = if let Some(meta) = transaction.clone().meta {
            meta
        } else {
            error!("Meta is malformed for transaction: {:?}", signature);
            return Err(anyhow::anyhow!(
                "Meta is malformed for transaction: {:?}",
                signature
            ));
        };

        if meta_original.status.is_err() {
            error!("Meta is malformed for transaction: {:?}", signature);
            return Err(anyhow::anyhow!(
                "Meta is malformed for transaction: {:?}",
                signature
            ));
        }

        let decoded_transaction = transaction
            .transaction
            .decode()
            .expect("Failed to decode transaction.");

        let meta_needed = transaction_metadata_from_original_meta(
            meta_original,
        )
        .expect("Error getting metadata from transaction original meta.");

        let transaction_update = Box::new(TransactionUpdate {
            signature,
            transaction: decoded_transaction.clone(),
            meta: meta_needed,
            is_vote: false,
            slot: encoded_transaction.slot,
            block_time: encoded_transaction.block_time,
        });

        let transaction_metadata =
            &(*transaction_update).clone().try_into().expect(
                "Failed to convert transaction update to transaction metadata.",
            );
        let instructions_with_metadata = extract_instructions_with_metadata(
            transaction_metadata,
            &transaction_update,
        )
        .expect("Failed to extract instructions with metadata.");
        let mint_details =
            extra_mint_details_from_tx_metadata(transaction_metadata);
        let nested_instructions: NestedInstructions =
            instructions_with_metadata.into();
        let mut swap_instruction: NestedInstruction =
            nested_instructions[outer_index].clone();
        if let Some(inner_index) = inner_index {
            swap_instruction =
                swap_instruction.inner_instructions[inner_index].clone();
        }

        let inner_instructions = swap_instruction.inner_instructions;

        let transfers: Vec<TokenTransferDetails> = SPL_TOKEN_TRANSFER_PROCESSOR
            .decode_token_transfer_with_vaults_from_nested_instructions(
                &inner_instructions,
                &mint_details,
            );
        Ok(transfers)
    }

    #[tokio::test]
    async fn test_buy_signature() {
        let signature = "538voMuFQKp3oE6Tu598R8kJN12sum2cGMxZBxrV2Vuip1TL4qdWaXiJ8u3yRxgJy9SFX4faP2zC83oDX68D2wuW";
        let mut vaults = HashSet::new();
        vaults
            .insert("xZtEgunCtNhMUbmPFGpGZCJ6oPzXCfQGbgXWjxhsQTM".to_string());
        vaults
            .insert("F6gGUPwvLeg4YEW6pXFowfX76dYPpG1PB51Vrm5Mc1C3".to_string());
        let outer_index = 0;
        let transfers = get_transaction(signature, outer_index, None)
            .await
            .expect("failed to get transaction with binary encoding");
        let DiffsResult {
            is_buy,
            price,
            swap_amount,
            ..
        } = process_token_transfers(&vaults, &transfers, 203.67).unwrap();
        let rounded_price = round_to_decimals(price, 5);
        assert!(rounded_price == 0.00035, "price: {}", rounded_price);
        let rounded_swap_amount = round_to_decimals(swap_amount, 4);
        assert!(
            rounded_swap_amount == 0.8618,
            "swap_amount: {}",
            rounded_swap_amount
        );
        assert!(!is_buy, "is_buy: {}", is_buy);
    }

    #[tokio::test]
    async fn test_multi_hop_by_signature() {
        let signature = "5f3jb13ZgqKBNvGSMC5wGgJvNa4bGBaVHSXXjWqMXHiXQUj8SEpCov9pMD6K4nXCGLxcpMLfgGJHmT5A24vC2sHd";
        // 2.1 - SolFi: Swap
        {
            let outer_index = 1;
            let inner_index = 0;
            let mut vaults = HashSet::new();
            vaults.insert(
                "5ep3LMR5gpCLD5KvSa9bnhR4R5Wm7HM7i1suP9u6ZvJT".to_string(),
            );
            vaults.insert(
                "3TokFuQgkkc6eLmafofNApdLkYpBvU1sZovyyScnQBD1".to_string(),
            );
            let transfers =
                get_transaction(signature, outer_index, Some(inner_index))
                    .await
                    .expect("failed to get transaction with binary encoding");

            let DiffsResult {
                is_buy,
                price,
                swap_amount,
                ..
            } = process_token_transfers(&vaults, &transfers, 203.67).unwrap();
            let rounded_price = round_to_decimals(price, 5);
            assert!(rounded_price == 1.36929, "price: {}", rounded_price);
            let rounded_swap_amount = round_to_decimals(swap_amount, 4);
            assert!(
                rounded_swap_amount == 101.835,
                "swap_amount: {}",
                rounded_swap_amount
            );
            assert!(is_buy, "is_buy: {}", is_buy);
        }

        // 2.5 - Meteora DLMM Program: swap
        {
            let outer_index: usize = 1;
            let inner_index = 2;
            let mut vaults = HashSet::new();
            vaults.insert(
                "5Ys4iNr3MVhXYdtoHtCjcYvMq34MjnkFynaxNihy71M4".to_string(),
            );
            vaults.insert(
                "2GHtKmEEEX2vwqD3btyNUUibhE3DvowojCpLH178t7Pk".to_string(),
            );
            let transfers =
                get_transaction(signature, outer_index, Some(inner_index))
                    .await
                    .expect("failed to get transaction with binary encoding");
            let DiffsResult {
                is_buy,
                price,
                swap_amount,
                ..
            } = process_token_transfers(&vaults, &transfers, 203.67).unwrap();
            let rounded_price = round_to_decimals(price, 5);
            assert!(rounded_price == 1.36933, "price: {}", rounded_price);
            let rounded_swap_amount = round_to_decimals(swap_amount, 4);
            assert!(
                rounded_swap_amount == 101.8378,
                "swap_amount: {}",
                rounded_swap_amount
            );
            assert!(!is_buy, "is_buy: {}", is_buy);
        }
    }

    #[tokio::test]
    async fn test_multi_dexes_by_signature() {
        let signature = "3m4LERWUekW7im8rgu8QgpSJA8a9yEYL3gDvorbd5YpkXarrL3PGoVmyFyQzd1Pw9oZiQy2LPUjaG8Xr4p433kwn";
        // 3.2 - Raydium Liquidity Pool V4: raydium:swap
        {
            let outer_index = 2;
            let inner_index = 1;
            let mut vaults = HashSet::new();
            vaults.insert(
                "F6iWqisguZYprVwp916BgGR7d5ahP6Ev5E213k8y3MEb".to_string(),
            );
            vaults.insert(
                "7bxbfwXi1CY7zWUXW35PBMZjhPD27SarVuHaehMzR2Fn".to_string(),
            );
            let transfers =
                get_transaction(signature, outer_index, Some(inner_index))
                    .await
                    .expect("failed to get transaction with binary encoding");

            let DiffsResult {
                is_buy,
                price,
                swap_amount,
                ..
            } = process_token_transfers(&vaults, &transfers, 203.67).unwrap();
            let rounded_price = round_to_decimals(price, 5);
            assert!(rounded_price == 0.55458, "price: {}", rounded_price);
            let rounded_swap_amount = round_to_decimals(swap_amount, 4);
            assert!(
                rounded_swap_amount == 3327.4865,
                "swap_amount: {}",
                rounded_swap_amount
            );
            assert!(!is_buy, "is_buy: {}", is_buy);
        }

        // 3.6 - Meteora DLMM Program: swap
        {
            let outer_index = 2;
            let inner_index = 3;
            let mut vaults = HashSet::new();
            vaults.insert(
                "CMVrNeYhZnqdbZfQuijgcNvCfvTJN2WKvKSnt2q3HT6N".to_string(),
            );
            vaults.insert(
                "5EfbkfLpaz9mHeTN6FnhtN8DTdMGZDRURYcsQ1f1Utg6".to_string(),
            );
            let transfers =
                get_transaction(signature, outer_index, Some(inner_index))
                    .await
                    .expect("failed to get transaction with binary encoding");
            let DiffsResult {
                is_buy,
                price,
                swap_amount,
                ..
            } = process_token_transfers(&vaults, &transfers, 203.67).unwrap();
            let rounded_price = round_to_decimals(price, 5);
            assert!(rounded_price == 0.55378, "price: {}", rounded_price);
            let rounded_swap_amount = round_to_decimals(swap_amount, 4);
            assert!(
                rounded_swap_amount == 13290.7687,
                "swap_amount: {}",
                rounded_swap_amount
            );
            assert!(!is_buy, "is_buy: {}", is_buy);
        }

        // 3.11 - Meteora DLMM Program: swap
        {
            let outer_index = 2;
            let inner_index = 5;
            let mut vaults = HashSet::new();
            vaults.insert(
                "FwqN8rUaFiH749WjLsutLC5JmRUqwoL99fSqTKuUqKsj".to_string(),
            );
            vaults.insert(
                "5EfbkfLpaz9mHeTN6FnhtN8DTdMGZDRURYcsQ1f1Utg6".to_string(),
            );
            let transfers =
                get_transaction(signature, outer_index, Some(inner_index))
                    .await
                    .expect("failed to get transaction with binary encoding");

            let DiffsResult {
                is_buy,
                price,
                swap_amount,
                ..
            } = process_token_transfers(&vaults, &transfers, 203.67).unwrap();
            let rounded_price = round_to_decimals(price, 5);
            assert!(rounded_price == 0.07765, "price: {}", rounded_price);
            let rounded_swap_amount = round_to_decimals(swap_amount, 4);
            assert!(
                rounded_swap_amount == 3323.6510,
                "swap_amount: {}",
                rounded_swap_amount
            );
            assert!(is_buy, "is_buy: {}", is_buy);
        }

        // 3.16 - Raydium Liquidity Pool V4: raydium:swap
        {
            let outer_index = 2;
            let inner_index = 7;
            let mut vaults = HashSet::new();
            vaults.insert(
                "5N8nDGtftaaX2ixPurZasiJyQPYFqbbV5XAnkTVxHpc8".to_string(),
            );
            vaults.insert(
                "ANcLMBXC9jNkWUTekV1YpPiHwBp8konJsyCDvyKYXmqv".to_string(),
            );
            let transfers =
                get_transaction(signature, outer_index, Some(inner_index))
                    .await
                    .expect("failed to get transaction with binary encoding");
            let DiffsResult {
                is_buy,
                price,
                swap_amount,
                ..
            } = process_token_transfers(&vaults, &transfers, 203.67).unwrap();
            let rounded_price = round_to_decimals(price, 5);
            assert!(rounded_price == 0.07754, "price: {}", rounded_price);
            let rounded_swap_amount = round_to_decimals(swap_amount, 4);
            assert!(
                rounded_swap_amount == 13294.6041,
                "swap_amount: {}",
                rounded_swap_amount
            );
            assert!(is_buy, "is_buy: {}", is_buy);
        }
    }

    #[tokio::test]
    async fn test_sell_swap() {
        let vaults = HashSet::from([
            "GHs3Cs9J6NoX79Nr2KvR1Nnzm82R34Jmqh1A8Bb84zgc".to_string(),
            "4UKfPxrJGEXggv637xCbzethVUGtkv6vay5zCjDSg1Yb".to_string(),
        ]);
        let fee_adas = HashSet::from([
            "Bvtgim23rfocUzxVX9j9QFxTbBnH8JZxnaGLCEkXvjKS".to_string(),
        ]);
        let transfers = vec![
            TokenTransferDetails {
                amount: 2523000000,
                ui_amount: 2523.0,
                decimals: 6,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "G2gUder2Y934cm8ufSQxjbhjrfJsiBBAox1jgLqEDx75"
                    .to_string(),
                destination: "GHs3Cs9J6NoX79Nr2KvR1Nnzm82R34Jmqh1A8Bb84zgc"
                    .to_string(),
                mint: "2WZuixz3wohXbib7Ze2gRjVeGeESiMw9hsizDwbjM4YK"
                    .to_string(),
                source: "yAcYcbC9Qr9SBpeG9SbT1zAEFwHd8j6EFFWomjQjVtn"
                    .to_string(),
            },
            TokenTransferDetails {
                amount: 7229486,
                ui_amount: 0.007229486,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "ezWtvReswwZaEBThCnW23qtH5uANic2akGY7yh7vZR9"
                    .to_string(),
                destination: "6qxghyVLU7sVYhQn6JKziDqb2VMPuDS6Q6rGngnkXdxx"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "4UKfPxrJGEXggv637xCbzethVUGtkv6vay5zCjDSg1Yb"
                    .to_string(),
            },
            TokenTransferDetails {
                amount: 3624,
                ui_amount: 0.000003624,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "ezWtvReswwZaEBThCnW23qtH5uANic2akGY7yh7vZR9"
                    .to_string(),
                destination: "Bvtgim23rfocUzxVX9j9QFxTbBnH8JZxnaGLCEkXvjKS"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "4UKfPxrJGEXggv637xCbzethVUGtkv6vay5zCjDSg1Yb"
                    .to_string(),
            },
        ];
        let is_valid =
            is_valid_vault_transfer(&transfers[0], &vaults, Some(&fee_adas));
        assert!(is_valid, "token should be valid");

        let is_valid =
            is_valid_vault_transfer(&transfers[1], &vaults, Some(&fee_adas));
        assert!(is_valid, "wsol ix should be valid");

        let is_valid =
            is_valid_vault_transfer(&transfers[2], &vaults, Some(&fee_adas));
        assert!(!is_valid, "the fee ix should be invalid");
    }

    #[tokio::test]
    async fn test_buy_swap() {
        let vaults: HashSet<String> = HashSet::from([
            "GkcKiF8ku7e54A8NK4UPHW6rmoGfhMeiMHGPpn4yUTkG".to_string(),
            "39NaF7ehkzNcxXLq9WZdtQ18RFu1rVxs3oQR1a2safoT".to_string(),
        ]);
        let fee_adas = HashSet::from([
            "94qWNrtmfn42h3ZjUZwWvK1MEo9uVmmrBPd2hpNjYDjb".to_string(),
        ]);
        let transfers = vec![
            TokenTransferDetails {
                amount: 540059097867,
                ui_amount: 540059.097867,
                decimals: 6,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "B67fRazWFUd4DPgFgLjKXX9dC8vS1UMbXmzYo7YDAqeA"
                    .to_string(),
                destination: "9qr6mtX3fELoWGQJyVzHgxuQZptZhmHRMdgZNyGDZkjB"
                    .to_string(),
                mint: "2Y6GkQJR93PNL1iYwGcjggoaBRaeTM1p9pC7oCzTpump"
                    .to_string(),
                source: "GkcKiF8ku7e54A8NK4UPHW6rmoGfhMeiMHGPpn4yUTkG"
                    .to_string(),
            },
            TokenTransferDetails {
                authority: "4sDjn4xpDBzd2QiKKGqmprCxeSLaDygC5oijyLLo6qUX"
                    .to_string(),
                destination: "39NaF7ehkzNcxXLq9WZdtQ18RFu1rVxs3oQR1a2safoT"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "GHjM41KiTeTiRR2m42RQF4jSpho4C4KKSx4D1ZX7D3Qb"
                    .to_string(),
                amount: 501000002,
                decimals: 9,
                ui_amount: 0.501000002,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
            },
            TokenTransferDetails {
                amount: 250001,
                ui_amount: 0.000250001,
                decimals: 9,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    .to_string(),
                authority: "4sDjn4xpDBzd2QiKKGqmprCxeSLaDygC5oijyLLo6qUX"
                    .to_string(),
                destination: "94qWNrtmfn42h3ZjUZwWvK1MEo9uVmmrBPd2hpNjYDjb"
                    .to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                source: "GHjM41KiTeTiRR2m42RQF4jSpho4C4KKSx4D1ZX7D3Qb"
                    .to_string(),
            },
        ];

        let is_valid =
            is_valid_vault_transfer(&transfers[0], &vaults, Some(&fee_adas));
        assert!(is_valid, "token should be valid");

        let is_valid =
            is_valid_vault_transfer(&transfers[1], &vaults, Some(&fee_adas));
        assert!(is_valid, "wsol ix should be valid");
    }
}
