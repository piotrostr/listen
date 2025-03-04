use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error};

use crate::{
    db::ClickhouseDb, kv_store::RedisKVStore, message_queue::RedisMessageQueue,
    metrics::SwapMetrics, process_swap::process_swap,
};
use carbon_core::{
    error::CarbonResult, instruction::InstructionProcessorInputType,
    metrics::MetricsCollection, processor::Processor,
};
use carbon_raydium_amm_v4_decoder::instructions::RaydiumAmmV4Instruction;

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
        let (meta, instruction, _nested_instructions) = data;
        match &instruction.data {
            RaydiumAmmV4Instruction::SwapBaseIn(_)
            | RaydiumAmmV4Instruction::SwapBaseOut(_) => {
                self.spawn_swap_processor(&meta);
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
            .unwrap_or(500); // Default to 500

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
        meta: &carbon_core::instruction::InstructionMetadata,
    ) {
        debug!(
            "https://solscan.io/tx/{}",
            meta.transaction_metadata.signature
        );

        let message_queue = self.message_queue.clone();
        let kv_store = self.kv_store.clone();
        let tx_meta = meta.transaction_metadata.clone();
        let db = self.db.clone();
        let metrics = self.metrics.clone();
        let semaphore = self.semaphore.clone();

        metrics.increment_total_swaps();

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
                &tx_meta,
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
