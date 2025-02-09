use std::{collections::HashMap, sync::Arc};
use tracing::{debug, error};

use crate::{
    kv_store::RedisKVStore, message_queue::RedisMessageQueue, process_swap::process_swap,
    util::make_kv_store,
};
use carbon_core::{
    error::CarbonResult, instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_amm_v4_decoder::instructions::RaydiumAmmV4Instruction;
use solana_transaction_status::TransactionTokenBalance;

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
                debug!(
                    "https://solscan.io/tx/{}",
                    meta.transaction_metadata.signature
                );

                let message_queue = self.message_queue.clone();
                let kv_store = self.kv_store.clone();
                tokio::spawn(async move {
                    if let Err(e) = process_swap(
                        diffs,
                        meta.transaction_metadata.slot,
                        &message_queue,
                        &kv_store,
                    )
                    .await
                    {
                        error!(
                            "Error processing swap: {:#}\nError chain:\n{:?}\nTransaction: https://solscan.io/tx/{}", 
                            e,
                            e.chain().collect::<Vec<_>>(),
                            meta.transaction_metadata.signature
                        );
                    }
                });
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
    pub owner: String,
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
            pre_balances_map.insert(balance.mint, (amount, balance.owner));
        }
    }

    for balance in post_balances {
        if let Some(amount) = balance.ui_token_amount.ui_amount {
            post_balances_map.insert(balance.mint, (amount, balance.owner));
        }
    }

    for (mint, (pre_amount, owner)) in pre_balances_map {
        if let Some((post_amount, _)) = post_balances_map.get(&mint) {
            let diff = post_amount - pre_amount;
            diffs.push(Diff {
                mint,
                pre_amount,
                post_amount: *post_amount,
                diff,
                owner,
            });
        }
    }

    diffs
}
