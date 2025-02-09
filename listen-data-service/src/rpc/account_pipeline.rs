use crate::raydium_processor::RaydiumAmmV4AccountProcessor;
use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use carbon_log_metrics::LogMetrics;
use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_sdk::pubkey;
use std::sync::Arc;

use carbon_rpc_program_subscribe_datasource::{Filters, RpcProgramSubscribe};

pub fn make_raydium_rpc_accounts_pipeline() -> Result<Pipeline> {
    let pipeline = Pipeline::builder()
        .datasource(RpcProgramSubscribe::new(
            std::env::var("WS_URL")?,
            Filters::new(
                pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
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
        .metrics(Arc::new(LogMetrics::new()))
        .build()?;

    Ok(pipeline)
}
