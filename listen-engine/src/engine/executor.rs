use crate::engine::order::PrivyOrder;
use crate::privy::config::{PrivyConfig, PrivyConfigError};
use crate::privy::types::{
    SignAndSendEvmTransactionParams, SignAndSendEvmTransactionRequest,
    SignAndSendTransactionParams, SignAndSendTransactionRequest, SignAndSendTransactionResponse,
};
use crate::privy::util::create_http_client;
use anyhow::{anyhow, Result};

pub struct Executor {
    http_client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("[Executor] Initialize: {0}")]
    InitializeError(#[from] PrivyConfigError),

    #[error("[Executor] Failed to execute order: {0}")]
    ExecuteOrderError(String),

    #[error("[Executor] Failed to execute EVM transaction: {0}")]
    ExecuteEvmTransactionError(#[from] anyhow::Error),

    #[error("[Executor] Failed to execute Solana transaction: {0}")]
    ExecuteSolanaTransactionError(anyhow::Error),

    #[error("[Executor] HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
}

impl Executor {
    pub fn from_env() -> Result<Self, ExecutorError> {
        let privy_config = PrivyConfig::from_env().map_err(ExecutorError::InitializeError)?;
        let http_client = create_http_client(&privy_config);
        Ok(Self { http_client })
    }

    pub async fn execute_order(&self, order: PrivyOrder) -> Result<String, ExecutorError> {
        if order.is_solana() {
            if order.solana_transaction.is_none() {
                return Err(ExecutorError::ExecuteOrderError(
                    "Solana transaction required for Solana order".to_string(),
                ));
            }
            self.execute_solana_transaction(
                order.address,
                order.solana_transaction.unwrap(),
                order.caip2,
            )
            .await
        } else {
            if order.evm_transaction.is_none() {
                return Err(ExecutorError::ExecuteOrderError(
                    "EVM transaction required for EVM order".to_string(),
                ));
            }
            self.execute_evm_transaction(order.address, order.evm_transaction.unwrap(), order.caip2)
                .await
        }
    }

    async fn execute_evm_transaction(
        &self,
        address: String,
        transaction: serde_json::Value,
        caip2: String,
    ) -> Result<String, ExecutorError> {
        tracing::info!(?address, "Executing EVM transaction");
        let request = SignAndSendEvmTransactionRequest {
            address,
            chain_type: "ethereum".to_string(),
            method: "eth_sendTransaction".to_string(),
            caip2,
            params: SignAndSendEvmTransactionParams { transaction },
        };

        let response = self
            .http_client
            .post("https://auth.privy.io/api/v1/wallets/rpc")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ExecutorError::ExecuteEvmTransactionError(anyhow!(
                "Failed to send transaction: {}",
                response.text().await?
            )));
        }

        let result: SignAndSendTransactionResponse = response.json().await?;
        tracing::info!(
            ?result.method,
            ?result.data.hash,
            ?result.data.caip2,
            "Transaction sent",
        );
        Ok(result.data.hash)
    }

    async fn execute_solana_transaction(
        &self,
        address: String,
        transaction: String,
        caip2: String,
    ) -> Result<String, ExecutorError> {
        tracing::info!(?address, "Executing Solana transaction");
        let request = SignAndSendTransactionRequest {
            address,
            chain_type: "solana".to_string(),
            method: "signAndSendTransaction".to_string(),
            caip2,
            params: SignAndSendTransactionParams {
                transaction,
                encoding: "base64".to_string(),
            },
        };

        let response = self
            .http_client
            .post("https://api.privy.io/v1/wallets/rpc")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ExecutorError::ExecuteSolanaTransactionError(anyhow!(
                "Failed to sign transaction: {}",
                response.text().await?
            )));
        }

        let result: SignAndSendTransactionResponse = response.json().await?;
        tracing::info!(
            ?result.method,
            ?result.data.hash,
            ?result.data.caip2,
            "Transaction sent",
        );
        Ok(result.data.hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::caip2::Caip2;
    use crate::engine::constants::*;
    use crate::engine::executor::Executor;
    use crate::engine::order::PrivyOrder;

    #[tokio::test]
    async fn test_execute_order_eth() {
        let engine = Executor::from_env().unwrap();
        let order = PrivyOrder {
            user_id: "-".to_string(),
            address: TEST_ADDRESS_EVM.to_string(),
            caip2: Caip2::ARBITRUM.to_string(),
            evm_transaction: Some(serde_json::json!({
                "from": TEST_ADDRESS_EVM,
                "to": TEST_ADDRESS_EVM,
                "value": "0x111",
            })),
            solana_transaction: None,
        };
        let result = engine.execute_order(order).await.unwrap();
        assert_eq!(result.len(), 66);
    }
}
