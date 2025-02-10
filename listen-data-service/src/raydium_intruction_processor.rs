use std::sync::Arc;
use tracing::{debug, error};

use crate::{
    kv_store::RedisKVStore, message_queue::RedisMessageQueue,
    process_swap::process_swap, util::make_kv_store,
};
use carbon_core::{
    error::CarbonResult, instruction::InstructionProcessorInputType,
    metrics::MetricsCollection, processor::Processor,
};
use carbon_raydium_amm_v4_decoder::instructions::RaydiumAmmV4Instruction;

pub struct RaydiumAmmV4InstructionProcessor {
    pub kv_store: Arc<RedisKVStore>,
    pub message_queue: Arc<RedisMessageQueue>,
}

impl Default for RaydiumAmmV4InstructionProcessor {
    fn default() -> Self {
        Self::new()
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
            RaydiumAmmV4Instruction::SwapBaseIn(_) => {
                self.spawn_swap_processor(&meta, true);
            }
            RaydiumAmmV4Instruction::SwapBaseOut(_) => {
                self.spawn_swap_processor(&meta, false);
            }
            _ => {}
        }

        Ok(())
    }
}

impl RaydiumAmmV4InstructionProcessor {
    pub fn new() -> Self {
        Self {
            kv_store: make_kv_store().expect("Failed to create KV store"),
            message_queue: Arc::new(
                RedisMessageQueue::new(
                    &std::env::var("REDIS_URL").expect("REDIS_URL must be set"),
                )
                .expect("Failed to create message queue"),
            ),
        }
    }

    fn spawn_swap_processor(
        &self,
        meta: &carbon_core::instruction::InstructionMetadata,
        is_base_in: bool,
    ) {
        debug!(
            "https://solscan.io/tx/{}",
            meta.transaction_metadata.signature
        );

        let message_queue = self.message_queue.clone();
        let kv_store = self.kv_store.clone();
        let tx_meta = meta.transaction_metadata.clone();

        tokio::spawn(async move {
            if let Err(e) =
                process_swap(&tx_meta, &message_queue, &kv_store, is_base_in)
                    .await
            {
                error!(
                    "Error processing swap: {:#}\nError chain:\n{:?}\nTransaction: https://solscan.io/tx/{}", 
                    e,
                    e.chain().collect::<Vec<_>>(),
                    tx_meta.signature
                );
            }
        });
    }
}
