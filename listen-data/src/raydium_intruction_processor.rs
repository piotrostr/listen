use std::{collections::HashSet, sync::Arc};
use tokio::sync::Semaphore;
use tracing::{debug, error};

use crate::{
    db::ClickhouseDb, kv_store::RedisKVStore, message_queue::RedisMessageQueue,
    metrics::SwapMetrics, process_swap::process_swap,
};
use carbon_core::{
    deserialize::ArrangeAccounts,
    error::CarbonResult,
    instruction::{InstructionProcessorInputType, NestedInstruction},
    metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_amm_v4_decoder::instructions::{
    swap_base_in::SwapBaseIn, swap_base_out::SwapBaseOut,
    RaydiumAmmV4Instruction,
};

pub struct RaydiumAmmV4InstructionProcessor {
    pub kv_store: Arc<RedisKVStore>,
    pub message_queue: Arc<RedisMessageQueue>,
    pub db: Arc<ClickhouseDb>,
    pub metrics: Arc<SwapMetrics>,
    pub semaphore: Arc<Semaphore>,
}

#[async_trait::async_trait]
impl Processor for RaydiumAmmV4InstructionProcessor {
    type InputType = InstructionProcessorInputType<RaydiumAmmV4Instruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (meta, instruction, nested_instructions) = data;
        match &instruction.data {
            RaydiumAmmV4Instruction::SwapBaseIn(_) => {
                let accounts =
                    SwapBaseIn::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.pool_coin_token_account.to_string(),
                        accounts.pool_pc_token_account.to_string(),
                    ]);
                    self.spawn_swap_processor(
                        &vaults,
                        &meta,
                        &nested_instructions,
                    );
                }
            }
            RaydiumAmmV4Instruction::SwapBaseOut(_) => {
                let accounts =
                    SwapBaseOut::arrange_accounts(&instruction.accounts);
                if let Some(accounts) = accounts {
                    let vaults: HashSet<String> = HashSet::from([
                        accounts.pool_coin_token_account.to_string(),
                        accounts.pool_pc_token_account.to_string(),
                    ]);
                    self.spawn_swap_processor(
                        &vaults,
                        &meta,
                        &nested_instructions,
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl RaydiumAmmV4InstructionProcessor {
    pub fn new(
        kv_store: Arc<RedisKVStore>,
        message_queue: Arc<RedisMessageQueue>,
        db: Arc<ClickhouseDb>,
    ) -> Self {
        let concurrency_limit = std::env::var("SWAP_CONCURRENCY_LIMIT")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1000);

        Self {
            kv_store,
            message_queue,
            db,
            metrics: Arc::new(SwapMetrics::new()),
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
        }
    }

    fn spawn_swap_processor(
        &self,
        vaults: &HashSet<String>,
        meta: &carbon_core::instruction::InstructionMetadata,
        nested_instructions: &Vec<NestedInstruction>,
    ) {
        debug!(
            "https://solscan.io/tx/{}",
            meta.transaction_metadata.signature
        );

        let message_queue = self.message_queue.clone();
        let kv_store = self.kv_store.clone();
        let db = self.db.clone();
        let metrics = self.metrics.clone();
        let semaphore = self.semaphore.clone();

        let vaults = vaults.clone();
        let tx_meta = meta.transaction_metadata.clone();
        let nested_instructions = nested_instructions.clone();

        metrics.increment_total_swaps();
        metrics.increment_pending_swaps();

        tokio::spawn(async move {
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(e) => {
                    metrics.increment_failed_swaps();
                    error!("Failed to acquire semaphore permit: {}", e);
                    return;
                }
            };

            match process_swap(
                &vaults,
                &tx_meta,
                &nested_instructions,
                &message_queue,
                &kv_store,
                &db,
                &metrics,
            )
            .await
            {
                Ok(_) => {
                    metrics.increment_successful_swaps();
                }
                Err(e) => {
                    metrics.increment_failed_swaps();
                    error!(
                        ?e,
                        "Transaction: https://solscan.io/tx/{}",
                        tx_meta.signature
                    );
                }
            }
        });
    }
}
