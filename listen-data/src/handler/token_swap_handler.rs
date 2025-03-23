use crate::{
    db::ClickhouseDb, kv_store::RedisKVStore, message_queue::RedisMessageQueue,
    metrics::SwapMetrics, process_swap::process_swap,
};
use carbon_core::instruction::{InstructionMetadata, NestedInstruction};
use std::{collections::HashSet, sync::Arc};
use tracing::{debug, error};

pub struct TokenSwapHandler {
    pub kv_store: Arc<RedisKVStore>,
    pub message_queue: Arc<RedisMessageQueue>,
    pub db: Arc<ClickhouseDb>,
    pub metrics: Arc<SwapMetrics>,
}

impl TokenSwapHandler {
    pub fn new(
        kv_store: Arc<RedisKVStore>,
        message_queue: Arc<RedisMessageQueue>,
        db: Arc<ClickhouseDb>,
        metrics: Arc<SwapMetrics>,
    ) -> Self {
        Self {
            kv_store,
            message_queue,
            db,
            metrics,
        }
    }

    pub fn spawn_swap_processor(
        &self,
        vaults: &HashSet<String>,
        fee_adas: Option<&HashSet<String>>,
        meta: &InstructionMetadata,
        nested_instructions: &[NestedInstruction],
    ) {
        debug!(
            "https://solscan.io/tx/{}",
            meta.transaction_metadata.signature
        );

        let message_queue = self.message_queue.clone();
        let kv_store = self.kv_store.clone();
        let db = self.db.clone();
        let metrics = self.metrics.clone();

        let vaults = vaults.clone();
        let fee_adas = fee_adas.cloned();
        let tx_meta = meta.transaction_metadata.clone();
        let nested_instructions = nested_instructions.to_vec();

        metrics.increment_total_swaps();
        metrics.increment_pending_swaps();

        tokio::spawn(async move {
            match process_swap(
                &vaults,
                fee_adas.as_ref(),
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

#[cfg(test)]
pub mod test_swaps {
    pub use crate::{
        db::{ClickhouseDb, Database},
        diffs::{
            extra_mint_details_from_tx_metadata, DiffsError,
            TokenTransferDetails, SPL_TOKEN_TRANSFER_PROCESSOR,
        },
        handler::TokenSwapHandler,
        kv_store::RedisKVStore,
        message_queue::RedisMessageQueue,
        metrics::SwapMetrics,
        util::{make_db, make_kv_store, make_message_queue, make_rpc_client},
    };
    use anyhow::{anyhow, Result};
    use carbon_core::{
        datasource::TransactionUpdate,
        instruction::{NestedInstruction, NestedInstructions},
        transaction::TransactionMetadata,
        transformers::{
            extract_instructions_with_metadata,
            transaction_metadata_from_original_meta,
        },
    };
    use dotenv::dotenv;
    use solana_client::rpc_config::RpcTransactionConfig;
    use solana_sdk::{
        commitment_config::CommitmentConfig, signature::Signature,
    };
    use solana_transaction_status::UiTransactionEncoding;
    use std::{str::FromStr, sync::Arc};

    pub fn get_inner_token_transfers(
        transaction_metadata: &TransactionMetadata,
        nested_instructions: &[NestedInstruction],
    ) -> Result<Vec<TokenTransferDetails>, DiffsError> {
        let mint_details =
            extra_mint_details_from_tx_metadata(transaction_metadata);

        let transfers = SPL_TOKEN_TRANSFER_PROCESSOR
            .decode_token_transfer_with_vaults_from_nested_instructions(
                nested_instructions,
                &mint_details,
            );
        Ok(transfers)
    }

    pub async fn get_storages(
    ) -> (Arc<RedisKVStore>, Arc<RedisMessageQueue>, Arc<ClickhouseDb>) {
        let kv_store = make_kv_store().await.expect("Failed to make kv store");
        let message_queue = make_message_queue()
            .await
            .expect("Failed to make message queue");
        let db = make_db().await.expect("Failed to make db");
        (kv_store, message_queue, db)
    }

    pub async fn get_token_swap_handler() -> Arc<TokenSwapHandler> {
        let (kv_store, message_queue, db) = get_storages().await;
        let metrics = Arc::new(SwapMetrics::new());
        Arc::new(TokenSwapHandler::new(kv_store, message_queue, db, metrics))
    }

    pub async fn get_transaction_data(
        tx_hash: &str,
    ) -> Result<(Signature, Box<TransactionUpdate>, Box<TransactionMetadata>)>
    {
        dotenv().ok();
        let signature =
            Signature::from_str(tx_hash).expect("Failed to parse signature");
        let rpc_client = make_rpc_client().expect("Failed to make rpc client");
        let encoded_transaction = rpc_client
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Binary),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
            .await
            .expect("Failed to get transaction");

        let transaction = encoded_transaction.transaction;

        let meta_original = if let Some(meta) = transaction.clone().meta {
            meta
        } else {
            return Err(anyhow!(
                "Meta is malformed for transaction: {:?}",
                signature
            ));
        };

        if meta_original.status.is_err() {
            return Err(anyhow!("Transaction failed: {:?}", signature));
        }

        let decoded_transaction = transaction
            .transaction
            .decode()
            .ok_or_else(|| anyhow!("Failed to decode transaction"))?;

        let meta_needed =
            transaction_metadata_from_original_meta(meta_original)
                .map_err(|e| anyhow!("Error getting metadata: {}", e))?;

        let transaction_update = Box::new(TransactionUpdate {
            signature,
            transaction: decoded_transaction.clone(),
            meta: meta_needed,
            is_vote: false,
            slot: encoded_transaction.slot,
            block_time: encoded_transaction.block_time,
        });

        let transaction_metadata: TransactionMetadata =
            (*transaction_update).clone().try_into().expect(
                "Failed to convert transaction update to transaction metadata.",
            );

        Ok((
            signature,
            transaction_update,
            Box::new(transaction_metadata),
        ))
    }

    pub async fn get_nested_instruction(
        tx_hash: &str,
        outer_idx: usize,
        inner_idx: Option<usize>,
    ) -> Result<(
        NestedInstruction,
        Box<TransactionUpdate>,
        Box<TransactionMetadata>,
    )> {
        let (_, transaction_update, transaction_metadata) =
            get_transaction_data(tx_hash)
                .await
                .expect("Failed to get transaction data");
        let nested_instructions =
            extract_nested_instructions(&transaction_update)
                .expect("Failed to extract nested instructions");
        if outer_idx >= nested_instructions.len() {
            return Err(anyhow!("Outer index out of bounds"));
        }
        let mut nested_instruction = nested_instructions[outer_idx].clone();
        if let Some(inner_idx) = inner_idx {
            nested_instruction =
                nested_instruction.inner_instructions[inner_idx].clone()
        }
        Ok((nested_instruction, transaction_update, transaction_metadata))
    }

    pub fn extract_nested_instructions(
        transaction_update: &TransactionUpdate,
    ) -> Result<NestedInstructions> {
        let transaction_metadata =
            transaction_update.clone().try_into().map_err(|e| {
                anyhow!("Failed to convert transaction update: {}", e)
            })?;

        let instructions_with_metadata = extract_instructions_with_metadata(
            &transaction_metadata,
            transaction_update,
        )
        .map_err(|e| anyhow!("Failed to extract instructions: {}", e))?;

        Ok(instructions_with_metadata.into())
    }
}
