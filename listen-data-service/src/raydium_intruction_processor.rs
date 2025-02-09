use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};

use crate::{
    constants::{USDC_MINT_KEY_STR, WSOL_MINT_KEY_STR},
    kv_store::RedisKVStore,
    message_queue::{MessageQueue, PriceUpdate, RedisMessageQueue},
    metadata::get_token_metadata,
    sol_price_stream::SOL_PRICE_CACHE,
    util::make_kv_store,
};
use carbon_core::{
    error::CarbonResult, instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_amm_v4_decoder::instructions::RaydiumAmmV4Instruction;
use chrono::Utc;
use solana_transaction_status_client_types::TransactionTokenBalance;

pub struct RaydiumAmmV4InstructionProcessor {
    pub kv_store: Arc<RedisKVStore>,
    pub message_queue: Arc<RedisMessageQueue>,
}

impl RaydiumAmmV4InstructionProcessor {
    pub fn new() -> Self {
        Self {
            kv_store: make_kv_store().expect("Failed to create KV store"),
            message_queue: Arc::new(
                RedisMessageQueue::new("redis://127.0.0.1/")
                    .expect("Failed to create message queue"),
            ),
        }
    }

    async fn process_swap(&self, diffs: Vec<Diff>) -> Result<()> {
        // Only process swaps with exactly 2 tokens
        if diffs.len() != 2 {
            info!("Skipping swap with {} diffs", diffs.len());
            return Ok(());
        }

        let (token0, token1) = (&diffs[0], &diffs[1]);

        // Get absolute values of diffs since they're opposite signs
        let amount0 = token0.diff.abs();
        let amount1 = token1.diff.abs();

        // Determine which token is WSOL or USDC (if any)
        let (price, coin_mint) = match (token0.mint.as_str(), token1.mint.as_str()) {
            (WSOL_MINT_KEY_STR, other_mint) => {
                let sol_price = SOL_PRICE_CACHE.get_price().await;
                ((amount1 / amount0) * sol_price, other_mint)
            }
            (other_mint, WSOL_MINT_KEY_STR) => {
                let sol_price = SOL_PRICE_CACHE.get_price().await;
                ((amount0 / amount1) * sol_price, other_mint)
            }
            (USDC_MINT_KEY_STR, other_mint) => (amount1 / amount0, other_mint),
            (other_mint, USDC_MINT_KEY_STR) => (amount0 / amount1, other_mint),
            _ => return Ok(()), // Skip pairs without SOL or USDC
        };

        // Get metadata for the non-WSOL/USDC token
        let token_metadata = get_token_metadata(&self.kv_store, &coin_mint).await?;

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
        };

        info!("price_update: {:#?}", price_update);

        self.message_queue
            .publish_price_update(price_update)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Processor for RaydiumAmmV4InstructionProcessor {
    type InputType = InstructionProcessorInputType<RaydiumAmmV4Instruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (meta, instruction, _nested_instructions) = data;
        match &instruction.data {
            RaydiumAmmV4Instruction::SwapBaseIn(_) | RaydiumAmmV4Instruction::SwapBaseOut(_) => {
                let diffs = get_token_balance_diff(
                    meta.transaction_metadata.meta.pre_token_balances.unwrap(),
                    meta.transaction_metadata.meta.post_token_balances.unwrap(),
                );

                info!("diffs: {:#?}", diffs);

                if let Err(e) = self.process_swap(diffs).await {
                    error!("Error processing swap: {}", e);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Diff {
    pub mint: String,
    pub pre_amount: f64,
    pub post_amount: f64,
    pub diff: f64,
}

fn get_token_balance_diff(
    pre_balances: Vec<TransactionTokenBalance>,
    post_balances: Vec<TransactionTokenBalance>,
) -> Vec<Diff> {
    let mut diffs = Vec::new();
    let mut pre_balances_map = HashMap::new();
    let mut post_balances_map = HashMap::new();

    for balance in pre_balances {
        if let Some(amount) = balance.ui_token_amount.ui_amount {
            pre_balances_map.insert(balance.mint, amount);
        }
    }

    for balance in post_balances {
        if let Some(amount) = balance.ui_token_amount.ui_amount {
            post_balances_map.insert(balance.mint, amount);
        }
    }

    for (mint, pre_amount) in pre_balances_map {
        if let Some(&post_amount) = post_balances_map.get(&mint) {
            let diff = pre_amount - post_amount;
            diffs.push(Diff {
                mint,
                pre_amount,
                post_amount,
                diff,
            });
        }
    }

    diffs
}
