use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use carbon_log_metrics::LogMetrics;
use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;
use carbon_yellowstone_grpc_datasource::YellowstoneGrpcGeyserClient;
use solana_sdk::{pubkey, pubkey::Pubkey};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequestFilterAccounts,
    SubscribeRequestFilterTransactions,
};

use crate::{
    db::ClickhouseDb, kv_store::RedisKVStore, message_queue::RedisMessageQueue,
    raydium_intruction_processor::RaydiumAmmV4InstructionProcessor,
    util::must_get_env,
};

pub const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey =
    pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

pub fn make_raydium_geyser_instruction_pipeline(
    kv_store: Arc<RedisKVStore>,
    message_queue: Arc<RedisMessageQueue>,
    db: Arc<ClickhouseDb>,
) -> Result<Pipeline> {
    // Set up transaction filters to only process Raydium transactions
    let mut transaction_filters = HashMap::new();
    transaction_filters.insert(
        "raydium_transaction_filter".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: Some(false),
            account_include: vec![],
            account_exclude: vec![],
            account_required: vec![RAYDIUM_AMM_V4_PROGRAM_ID.to_string()],
            signature: None,
        },
    );

    // Create empty account filters since we only care about transactions
    let account_filters: HashMap<String, SubscribeRequestFilterAccounts> =
        HashMap::new();

    let pipeline = Pipeline::builder()
        .datasource(YellowstoneGrpcGeyserClient::new(
            must_get_env("GEYSER_URL"),
            Some(must_get_env("GEYSER_X_TOKEN")),
            Some(CommitmentLevel::Processed),
            account_filters,
            transaction_filters,
            Arc::new(RwLock::new(HashSet::new())),
        ))
        .metrics(Arc::new(LogMetrics::new()))
        .instruction(
            RaydiumAmmV4Decoder,
            RaydiumAmmV4InstructionProcessor::new(kv_store, message_queue, db),
        )
        .build()?;

    Ok(pipeline)
}
