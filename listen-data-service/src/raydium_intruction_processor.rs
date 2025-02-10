use std::sync::Arc;
use tracing::{debug, error};

use crate::{
    db::ClickhouseDb, kv_store::RedisKVStore, message_queue::RedisMessageQueue,
    process_swap::process_swap,
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
        Self {
            kv_store,
            message_queue,
            db,
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
        tokio::spawn(async move {
            if let Err(e) =
                process_swap(&tx_meta, &message_queue, &kv_store, &db).await
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
