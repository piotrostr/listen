use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use carbon_log_metrics::LogMetrics;
use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;
use carbon_rpc_transaction_crawler_datasource::{Filters, RpcTransactionCrawler};
use solana_sdk::pubkey;
use std::{sync::Arc, time::Duration};

use crate::raydium_intruction_processor::RaydiumAmmV4InstructionProcessor;

pub fn make_raydium_instruction_pipeline() -> Result<Pipeline> {
    let pipeline = Pipeline::builder()
        .datasource(RpcTransactionCrawler::new(
            std::env::var("RPC_URL")?,
            pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
            1,
            Duration::from_secs(1),
            Filters::new(None, None, None),
            None,
            1,
        ))
        .metrics(Arc::new(LogMetrics::new()))
        .instruction(RaydiumAmmV4Decoder, RaydiumAmmV4InstructionProcessor::new())
        .build()?;

    Ok(pipeline)
}
