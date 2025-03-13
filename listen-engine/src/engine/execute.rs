use std::sync::Arc;

use crate::engine::{
    order::{swap_order_to_transaction, SwapOrder, SwapOrderTransaction},
    retry::retry_with_backoff,
    Engine, EngineError,
};
use blockhash_cache::{inject_blockhash_into_encoded_tx, BLOCKHASH_CACHE};
use evm_approvals::{caip2_to_chain_id, create_approval_transaction, get_allowance};
use privy::{tx::PrivyTransaction, Privy};

impl Engine {
    pub async fn execute_order(
        &self,
        order: &SwapOrder,
        user_id: &str,
        wallet_address: Option<String>,
        pubkey: Option<String>,
    ) -> Result<String, EngineError> {
        if wallet_address.is_none() && order.is_evm() {
            return Err(EngineError::EVMWalletNotAvailable);
        }
        if pubkey.is_none() && order.is_solana() {
            return Err(EngineError::SolanaWalletNotAvailable);
        }
        let address = match order.is_evm() {
            true => wallet_address.clone().unwrap(),
            false => pubkey.clone().unwrap(),
        };
        let mut privy_transaction = PrivyTransaction {
            user_id: user_id.to_string(),
            address,
            from_chain_caip2: order.from_chain_caip2.clone(),
            to_chain_caip2: order.to_chain_caip2.clone(),
            evm_transaction: None,
            solana_transaction: None,
        };
        let lifi_api_key: Option<String> = match std::env::var("LIFI_API_KEY") {
            Ok(val) => Some(val),
            Err(_) => None,
        };

        match swap_order_to_transaction(
            order,
            &lifi::LiFi::new(lifi_api_key),
            wallet_address.clone(),
            pubkey.clone(),
        )
        .await
        .map_err(EngineError::SwapOrderError)?
        {
            SwapOrderTransaction::Evm(transaction) => {
                let spender_address = transaction["to"].as_str().unwrap();
                ensure_approvals(
                    spender_address,
                    order,
                    &privy_transaction,
                    self.privy.clone(),
                )
                .await?;
                privy_transaction.evm_transaction = Some(transaction);
                match self
                    .privy
                    .execute_transaction(privy_transaction.clone())
                    .await
                {
                    Ok(transaction_hash) => Ok(transaction_hash),
                    Err(e) => {
                        tracing::error!(transaction = ?privy_transaction, ?order, error = %e, "Failed to execute evm order");
                        Err(EngineError::TransactionError(e))
                    }
                }
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

                // Execute Solana transaction with retry
                execute_solana_transaction_with_retry(&privy_transaction, self.privy.clone(), order)
                    .await
            }
        }
    }
}

pub async fn ensure_approvals(
    spender_address: &str,
    order: &SwapOrder,
    privy_transaction: &PrivyTransaction,
    privy: Arc<Privy>,
) -> Result<(), EngineError> {
    let allowance = get_allowance(
        &order.input_token,
        &privy_transaction.address,
        spender_address,
        caip2_to_chain_id(&order.from_chain_caip2).unwrap(),
    )
    .await
    .map_err(EngineError::ApprovalsError)?;
    if allowance < order.amount.parse::<u128>().unwrap() {
        let approval_transaction = create_approval_transaction(
            &order.input_token,
            spender_address,
            &privy_transaction.address,
            caip2_to_chain_id(&order.from_chain_caip2).unwrap(),
        )
        .await
        .map_err(EngineError::ApprovalsError)?;
        let mut approval_privy_tx = privy_transaction.clone();
        approval_privy_tx.evm_transaction = Some(approval_transaction);
        privy
            .execute_transaction(approval_privy_tx)
            .await
            .map_err(EngineError::TransactionError)?;
    }
    Ok(())
}

pub async fn execute_solana_transaction_with_retry(
    privy_transaction: &PrivyTransaction,
    privy: Arc<Privy>,
    order: &SwapOrder,
) -> Result<String, EngineError> {
    retry_with_backoff("execute_solana_transaction", || {
        let privy_tx = privy_transaction.clone();
        let privy_clone = privy.clone();

        async move {
            match privy_clone.execute_transaction(privy_tx).await {
                Ok(transaction_hash) => Ok(transaction_hash),
                Err(e) => {
                    tracing::warn!(
                        ?order,
                        error = %e,
                        "Solana transaction execution failed, will retry"
                    );
                    Err(EngineError::TransactionError(e))
                }
            }
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use privy::{config::PrivyConfig, tx::PrivyTransaction, Privy};

    #[tokio::test]
    // TODO works but doesnt go through fully
    async fn test_tx_with_approvals() {
        let privy_tx = PrivyTransaction {
            user_id: "did:privy:cm6cxky3i00ondmuatkemmffm".to_string(),
            address: "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770".to_string(),
            from_chain_caip2: "eip155:42161".to_string(),
            to_chain_caip2: "eip155:8453".to_string(),
            evm_transaction: Some(serde_json::json!({
                "chain_id": 42161,
                "data": "0xae3285900000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000020076ab8ae28e0bdf4db3561cfacd5e08c72180f6e75c21b4db3f44018a7fa0a974000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000ccc48877a33a2c14e40c82da843cf4c607abf77000000000000000000000000000000000000000000000000000000000004c4b40000000000000000000000000000000000000000000000000000000000000210500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000572656c617900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000086c6966692d6170690000000000000000000000000000000000000000000000007b59061c373d90a64dc24aacba591b79d8c194cc2db0b22b6a6c25a0d2454e83000000000000000000000000ccc48877a33a2c14e40c82da843cf4c607abf770000000000000000000000000532f27101965dd16442e59d40670faf5ebb142e4000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000417e51c1fc390dd5f9f065a7f87a4aa658c70d382ba748837e87e725012ab8fd362f665152cac0e1bd0939d14682ccc43d5072f6a1ff352e39e69ebc9332a4688d1c00000000000000000000000000000000000000000000000000000000000000",
                "from": "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770",
                "gas_limit": "0x2a0225", // TODO the error occurs when gas_limit is given as gas and gas_price is gasPrice, chainId is accepted both
                "gas_price": "0x989680",
                "to": "0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE",
                "value": "0x0",
            })),
            solana_transaction: None,
        };

        dotenv::dotenv().ok();
        let result = Privy::new(PrivyConfig::from_env().unwrap())
            .execute_transaction(privy_tx)
            .await;
        println!("Result: {:#?}", result);
    }
}
