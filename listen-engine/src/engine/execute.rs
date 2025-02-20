use crate::engine::{
    order::{swap_order_to_transaction, SwapOrder, SwapOrderTransaction},
    pipeline::Status,
    Engine, EngineError, Pipeline,
};
use blockhash_cache::{inject_blockhash_into_encoded_tx, BLOCKHASH_CACHE};
use privy::tx::PrivyTransaction;
use uuid::Uuid;

impl Engine {
    pub async fn execute_order(
        &self,
        pipeline: &mut Pipeline,
        step_id: Uuid,
        order: &SwapOrder,
    ) -> Result<(), EngineError> {
        // Execute transaction first
        let address = match order.is_evm() {
            true => pipeline.wallet_address.clone(),
            false => pipeline.pubkey.clone(),
        };
        let mut privy_transaction = PrivyTransaction {
            user_id: pipeline.user_id.clone(),
            address,
            from_chain_caip2: order.from_chain_caip2.clone(),
            to_chain_caip2: order.to_chain_caip2.clone(),
            evm_transaction: None,
            solana_transaction: None,
        };

        let transaction_result = match swap_order_to_transaction(
            order,
            &lifi::LiFi::new(None),
            &pipeline.wallet_address,
            &pipeline.pubkey,
        )
        .await
        .map_err(EngineError::SwapOrderError)?
        {
            SwapOrderTransaction::Evm(transaction) => {
                privy_transaction.evm_transaction = Some(transaction);
                self.privy.execute_transaction(privy_transaction).await
            }
            SwapOrderTransaction::Solana(transaction) => {
                let latest_blockhash = BLOCKHASH_CACHE
                    .get_blockhash()
                    .await
                    .map_err(EngineError::BlockhashCacheError)?;
                let fresh_blockhash_tx =
                    inject_blockhash_into_encoded_tx(&transaction, &latest_blockhash.to_string())
                        .map_err(EngineError::InjectBlockhashError)?;
                privy_transaction.solana_transaction = Some(fresh_blockhash_tx);
                self.privy.execute_transaction(privy_transaction).await
            }
        };

        // Update pipeline state after transaction execution
        if let Some(step) = pipeline.steps.get_mut(&step_id) {
            match transaction_result {
                Ok(_) => {
                    step.status = Status::Completed;
                    pipeline.current_steps = step.next_steps.clone();
                    self.redis
                        .save_pipeline(pipeline)
                        .await
                        .map_err(EngineError::SavePipelineError)?;
                }
                Err(e) => {
                    step.status = Status::Failed;
                    pipeline.status = Status::Failed;
                    self.redis
                        .save_pipeline(pipeline)
                        .await
                        .map_err(EngineError::SavePipelineError)?;
                    return Err(EngineError::TransactionError(e));
                }
            }
        }

        Ok(())
    }
}
