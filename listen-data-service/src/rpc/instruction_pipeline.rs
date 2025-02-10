use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use carbon_log_metrics::LogMetrics;
use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;
use carbon_rpc_transaction_crawler_datasource::{
    Filters, RpcTransactionCrawler,
};
use solana_sdk::pubkey;
use std::{sync::Arc, time::Duration};

use crate::{
    db::ClickhouseDb, kv_store::RedisKVStore, message_queue::RedisMessageQueue,
    raydium_intruction_processor::RaydiumAmmV4InstructionProcessor,
};

pub fn make_raydium_rpc_instruction_pipeline(
    kv_store: Arc<RedisKVStore>,
    message_queue: Arc<RedisMessageQueue>,
    db: Arc<ClickhouseDb>,
) -> Result<Pipeline> {
    let pipeline = Pipeline::builder()
        .datasource(RpcTransactionCrawler::new(
            std::env::var("RPC_URL")?,
            pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
            500,
            Duration::from_secs(1),
            Filters::new(None, None, None),
            None,
            100,
        ))
        .metrics(Arc::new(LogMetrics::new()))
        .instruction(
            RaydiumAmmV4Decoder,
            RaydiumAmmV4InstructionProcessor::new(kv_store, message_queue, db),
        )
        .build()?;

    Ok(pipeline)
}
