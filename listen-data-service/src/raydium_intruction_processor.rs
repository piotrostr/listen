use std::{collections::HashMap, sync::Arc};

use crate::{
    constants::{USDC_MINT_KEY_STR, WSOL_MINT_KEY_STR},
    kv_store::RedisKVStore,
    util::make_kv_store,
};
use carbon_core::{
    error::CarbonResult, instruction::InstructionProcessorInputType, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_amm_v4_decoder::instructions::RaydiumAmmV4Instruction;
use solana_transaction_status_client_types::TransactionTokenBalance;

pub struct RaydiumAmmV4InstructionProcessor {
    kv_store: Arc<RedisKVStore>,
}

impl RaydiumAmmV4InstructionProcessor {
    pub fn new() -> Self {
        Self {
            kv_store: make_kv_store().expect("Failed to create KV store"),
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
                println!(
                    "https://solscan.io/tx/{}",
                    meta.transaction_metadata.signature.to_string()
                );
                let diffs = get_token_balance_diff(
                    meta.transaction_metadata.meta.pre_token_balances.unwrap(),
                    meta.transaction_metadata.meta.post_token_balances.unwrap(),
                );
                let swapped_tokens = diffs
                    .iter()
                    .map(|diff| diff.mint.as_str())
                    .collect::<Vec<&str>>();
                if swapped_tokens.contains(&WSOL_MINT_KEY_STR)
                    && swapped_tokens.contains(&USDC_MINT_KEY_STR)
                {
                    for diff in diffs {
                        match self.kv_store.get_metadata(&diff.mint).await {
                            Ok(Some(metadata)) => {
                                println!(
                                    "{}: {} ({} -> {})",
                                    metadata.mpl.name, diff.diff, diff.pre_amount, diff.post_amount
                                );
                            }
                            _ => {
                                println!(
                                    "{}: {} ({} -> {})",
                                    diff.mint, diff.diff, diff.pre_amount, diff.post_amount
                                );
                            }
                        }
                    }
                }
                println!("--------------------------------");
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Diff {
    mint: String,
    pre_amount: f64,
    post_amount: f64,
    diff: f64,
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
