use crate::{
    types::{
        SignAndSendEvmTransactionParams, SignAndSendEvmTransactionRequest,
        SignAndSendTransactionParams, SignAndSendTransactionRequest,
        SignAndSendTransactionResponse,
    },
    Privy,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyTransaction {
    pub user_id: String,
    pub address: String,
    pub from_chain_caip2: String,
    pub to_chain_caip2: String,
    pub evm_transaction: Option<serde_json::Value>,
    pub solana_transaction: Option<String>, // base64
}

impl PrivyTransaction {
    pub fn is_solana(&self) -> bool {
        self.from_chain_caip2.starts_with("solana")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PrivyTransactionError {
    #[error("[Privy] Failed to execute transaction: {0}")]
    ExecuteTransactionError(String),

    #[error("[Privy] Failed to execute EVM transaction: {0}")]
    ExecuteEvmTransactionError(#[from] anyhow::Error),

    #[error("[Privy] Failed to execute Solana transaction: {0}")]
    ExecuteSolanaTransactionError(anyhow::Error),

    #[error("[Privy] HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
}

impl Privy {
    pub async fn execute_transaction(
        &self,
        transaction: PrivyTransaction,
    ) -> Result<String, PrivyTransactionError> {
        if transaction.is_solana() {
            if transaction.solana_transaction.is_none() {
                return Err(PrivyTransactionError::ExecuteTransactionError(
                    "Solana transaction required for Solana transaction".to_string(),
                ));
            }
            self.execute_solana_transaction(
                transaction.address,
                transaction.solana_transaction.unwrap(),
                transaction.from_chain_caip2,
            )
            .await
        } else {
            if transaction.evm_transaction.is_none() {
                return Err(PrivyTransactionError::ExecuteTransactionError(
                    "EVM transaction required for EVM order".to_string(),
                ));
            }
            self.execute_evm_transaction(
                transaction.address,
                transaction.evm_transaction.unwrap(),
                transaction.from_chain_caip2,
            )
            .await
        }
    }

    pub async fn execute_evm_transaction(
        &self,
        address: String,
        transaction: serde_json::Value,
        caip2: String,
    ) -> Result<String, PrivyTransactionError> {
        tracing::info!(?address, "Executing EVM transaction");
        let request = SignAndSendEvmTransactionRequest {
            address,
            chain_type: "ethereum".to_string(),
            method: "eth_sendTransaction".to_string(),
            caip2,
            params: SignAndSendEvmTransactionParams { transaction },
        };

        let response = self
            .client
            .post("https://auth.privy.io/api/v1/wallets/rpc")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PrivyTransactionError::ExecuteEvmTransactionError(anyhow!(
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

    pub async fn execute_solana_transaction(
        &self,
        address: String,
        transaction: String,
        caip2: String,
    ) -> Result<String, PrivyTransactionError> {
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
            .client
            .post("https://api.privy.io/v1/wallets/rpc")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PrivyTransactionError::ExecuteSolanaTransactionError(
                anyhow!("Failed to sign transaction: {}", response.text().await?),
            ));
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
    use crate::caip2::Caip2;
    use crate::config::PrivyConfig;

    use super::*;

    const TEST_ADDRESS_EVM: &str = "0x123"; // fill in

    #[tokio::test]
    #[ignore = "change the TEST_ADDRESS_EVM based on your environment before running"]
    async fn test_execute_order_eth() {
        let privy = Privy::new(PrivyConfig::from_env().unwrap());
        let privy_transaction = PrivyTransaction {
            user_id: "-".to_string(),
            address: TEST_ADDRESS_EVM.to_string(),
            from_chain_caip2: Caip2::ARBITRUM.to_string(),
            to_chain_caip2: Caip2::ARBITRUM.to_string(),
            evm_transaction: Some(serde_json::json!({
                "from": TEST_ADDRESS_EVM,
                "to": TEST_ADDRESS_EVM,
                "value": "0x111",
            })),
            solana_transaction: None,
        };
        let result = privy.execute_transaction(privy_transaction).await.unwrap();
        assert_eq!(result.len(), 66);
    }
}
