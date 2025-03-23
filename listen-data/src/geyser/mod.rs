use anyhow::Result;
use carbon_core::pipeline::{Pipeline, ShutdownStrategy};
use carbon_log_metrics::LogMetrics;
use carbon_meteora_dlmm_decoder::MeteoraDlmmDecoder;
use carbon_orca_whirlpool_decoder::OrcaWhirlpoolDecoder;
use carbon_pump_swap_decoder::PumpSwapDecoder;
use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;
use carbon_raydium_clmm_decoder::RaydiumClmmDecoder;
use carbon_raydium_cpmm_decoder::RaydiumCpmmDecoder;
use carbon_yellowstone_grpc_datasource::YellowstoneGrpcGeyserClient;
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
    constants::{
        METEORA_DLMM_PROGRAM_ID, PUMP_SWAP_PROGRAM_ID,
        RAYDIUM_AMM_V4_PROGRAM_ID, RAYDIUM_CLMM_PROGRAM_ID,
        RAYDIUM_CPMM_PROGRAM_ID, WHIRLPOOLS_PROGRAM_ID,
    },
    db::ClickhouseDb,
    handler::TokenSwapHandler,
    kv_store::RedisKVStore,
    message_queue::RedisMessageQueue,
    metrics::SwapMetrics,
    processor::{
        MeteoraDlmmInstructionProcessor, OcraWhirlpoolInstructionProcessor,
        PumpAmmInstructionProcessor, RaydiumAmmV4InstructionProcessor,
        RaydiumClmmInstructionProcessor, RaydiumCpmmInstructionProcessor,
    },
    util::must_get_env,
};

pub fn make_geyser_pipeline(
    kv_store: Arc<RedisKVStore>,
    message_queue: Arc<RedisMessageQueue>,
    db: Arc<ClickhouseDb>,
    metrics: Arc<SwapMetrics>,
) -> Result<Pipeline> {
    let mut transaction_filters = HashMap::new();
    // TODO support TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb (other token program)
    transaction_filters.insert(
        "swap_transaction_filter".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: Some(false),
            account_include: vec![
                RAYDIUM_AMM_V4_PROGRAM_ID.to_string(),
                RAYDIUM_CLMM_PROGRAM_ID.to_string(),
                RAYDIUM_CPMM_PROGRAM_ID.to_string(),
                METEORA_DLMM_PROGRAM_ID.to_string(),
                WHIRLPOOLS_PROGRAM_ID.to_string(),
                PUMP_SWAP_PROGRAM_ID.to_string(),
            ],
            account_exclude: vec![],
            account_required: vec![],
            signature: None,
        },
    );

    let token_swap_handler =
        Arc::new(TokenSwapHandler::new(kv_store, message_queue, db, metrics));

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
        .shutdown_strategy(ShutdownStrategy::Immediate)
        .instruction(
            RaydiumAmmV4Decoder,
            RaydiumAmmV4InstructionProcessor::new(token_swap_handler.clone()),
        )
        .instruction(
            RaydiumCpmmDecoder,
            RaydiumCpmmInstructionProcessor::new(token_swap_handler.clone()),
        )
        .instruction(
            MeteoraDlmmDecoder,
            MeteoraDlmmInstructionProcessor::new(token_swap_handler.clone()),
        )
        .instruction(
            OrcaWhirlpoolDecoder,
            OcraWhirlpoolInstructionProcessor::new(token_swap_handler.clone()),
        )
        .instruction(
            RaydiumClmmDecoder,
            RaydiumClmmInstructionProcessor::new(token_swap_handler.clone()),
        )
        .instruction(
            PumpSwapDecoder,
            PumpAmmInstructionProcessor::new(token_swap_handler.clone()),
        )
        .build()?;

    Ok(pipeline)
}
