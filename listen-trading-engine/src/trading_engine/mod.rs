mod caip2;
mod constants;
mod order;
mod privy_config;
mod types;
mod util;

use self::order::Order;
use self::privy_config::PrivyConfig;
use self::types::{
    SignAndSendTransactionParams, SignAndSendTransactionRequest, SignAndSendTransactionResponse,
};
use self::util::create_http_client;
use anyhow::{anyhow, Result};
use types::{SignAndSendEvmTransactionParams, SignAndSendEvmTransactionRequest};

pub struct TradingEngine {
    http_client: reqwest::Client,
}

impl TradingEngine {
    pub fn from_env() -> Result<Self> {
        let privy_config = PrivyConfig::from_env()?;
        let http_client = create_http_client(&privy_config);
        Ok(Self { http_client })
    }

    pub async fn execute_order(&self, order: Order) -> Result<String> {
        if order.is_solana() {
            if order.solana_transaction.is_none() {
                return Err(anyhow!("Solana transaction required for Solana order"));
            }
            self.execute_solana_transaction(
                order.address,
                order.solana_transaction.unwrap(),
                order.caip2,
            )
            .await
        } else {
            if order.evm_transaction.is_none() {
                return Err(anyhow!("EVM transaction required for EVM order"));
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
    ) -> Result<String> {
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
            return Err(anyhow!(
                "Failed to send transaction: {}",
                response.text().await?
            ));
        }

        let result: SignAndSendTransactionResponse = response.json().await?;
        tracing::info!(
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
    ) -> Result<String> {
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
            return Err(anyhow!(
                "Failed to sign transaction: {}",
                response.text().await?
            ));
        }

        let result: SignAndSendTransactionResponse = response.json().await?;
        tracing::info!(
            ?result.data.hash,
            ?result.data.caip2,
            "Transaction sent",
        );
        Ok(result.data.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caip2::Caip2;

    #[tokio::test]
    async fn test_execute_order_eth() {
        let engine = TradingEngine::from_env().unwrap();
        let order = Order {
            user_id: "-".to_string(),
            address: constants::TEST_ADDRESS_EVM.to_string(),
            caip2: Caip2::ARBITRUM.to_string(),
            condition: super::order::Condition {},
            evm_transaction: Some(serde_json::json!({
                "from": constants::TEST_ADDRESS_EVM,
                "to": constants::TEST_ADDRESS_EVM,
                "value": "0x111",
            })),
            solana_transaction: None,
        };
        let result = engine.execute_order(order).await.unwrap();
        assert_eq!(result.len(), 66);
    }
}
