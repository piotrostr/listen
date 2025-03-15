use crate::{
    constants::RAYDIUM_AMM_V4_PROGRAM_ID,
    processor::RaydiumAmmV4AccountProcessor, util::must_get_env,
};
use anyhow::Result;
use carbon_core::pipeline::{Pipeline, ShutdownStrategy};
use carbon_log_metrics::LogMetrics;
use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;
use carbon_rpc_program_subscribe_datasource::{Filters, RpcProgramSubscribe};
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{
    RpcAccountInfoConfig, RpcProgramAccountsConfig,
};
use std::sync::Arc;

pub fn make_raydium_rpc_accounts_pipeline() -> Result<Pipeline> {
    let pipeline = Pipeline::builder()
        .datasource(RpcProgramSubscribe::new(
            must_get_env("WS_URL"),
            Filters::new(
                RAYDIUM_AMM_V4_PROGRAM_ID,
                Some(RpcProgramAccountsConfig {
                    filters: None,
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ),
        ))
        .account(RaydiumAmmV4Decoder, RaydiumAmmV4AccountProcessor::new())
        .shutdown_strategy(ShutdownStrategy::Immediate)
        .metrics(Arc::new(LogMetrics::new()))
        .build()?;

    Ok(pipeline)
}
