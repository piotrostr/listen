use async_trait::async_trait;
use ethers::utils::hex::ToHexExt;
use ethers::{providers::Middleware, signers::LocalWallet};
use privy::caip2::Caip2;

use crate::evm::util::make_ethers_provider;

use super::TransactionSigner;

#[async_trait]
impl TransactionSigner for LocalWallet {
    fn address(&self) -> Option<String> {
        let addr = ethers::signers::Signer::address(self);
        Some(format!("0x{}", addr.as_bytes().encode_hex()))
    }

    #[cfg(feature = "hype")]
    async fn secp256k1_sign(
        &self,
        message: ethers::types::H256,
    ) -> std::result::Result<
        ethers::types::Signature,
        hyperliquid_rust_sdk::Error,
    > {
        self.sign_hash(message)
            .map_err(|e| hyperliquid_rust_sdk::Error::Wallet(e.to_string()))
    }

    // FIXME this is automatically picking the chain ID as arbitrum if
    // unspecified
    // if used uncarefully, might lead to loss of funds by transfering over invalid network
    async fn sign_and_send_evm_transaction(
        &self,
        tx: alloy::rpc::types::TransactionRequest,
    ) -> anyhow::Result<String> {
        let caip2 =
            tx.chain_id.map_or(Caip2::ARBITRUM.to_string(), |chain_id| {
                Caip2::from_chain_id(chain_id).to_string()
            });
        self.sign_and_send_json_evm_transaction(
            serde_json::to_value(tx)?,
            Some(caip2),
        )
        .await
    }

    async fn sign_and_send_json_evm_transaction(
        &self,
        tx: serde_json::Value,
        caip2: Option<String>,
    ) -> anyhow::Result<String> {
        let chain_id = caip2
            .map_or(Caip2::to_chain_id(Caip2::ARBITRUM), |caip2| {
                Caip2::to_chain_id(&caip2)
            });
        let provider = make_ethers_provider(chain_id)?;
        let client =
            ethers::middleware::SignerMiddleware::new_with_provider_chain(
                provider,
                self.clone(),
            )
            .await?;

        let tx: ethers::types::TransactionRequest =
            serde_json::from_value(tx)?;
        let pending_tx =
            client.send_transaction(tx, None).await.map_err(|e| {
                anyhow::anyhow!("Failed to broadcast transaction: {}", e)
            })?;

        let receipt = pending_tx
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to get transaction receipt: {}", e)
            })?
            .ok_or_else(|| {
                anyhow::anyhow!("Transaction receipt not found")
            })?;

        if receipt.status.unwrap_or_default().as_u64() == 0 {
            return Err(anyhow::anyhow!("Transaction reverted"));
        }

        Ok(format!("0x{:x}", receipt.transaction_hash))
    }
}

#[cfg(test)]
mod tests {
    use ethers::{signers::LocalWallet, types::H256};

    use crate::signer::TransactionSigner;

    #[tokio::test]
    async fn test_secp256k1_sign_local_wallet() {
        let message = H256::default();
        let private_key = std::env::var("ETHEREUM_PRIVATE_KEY").unwrap();
        let signer: LocalWallet = private_key.try_into().unwrap();
        let _ = signer.secp256k1_sign(message).await.unwrap();
    }

    #[tokio::test]
    async fn test_sign_and_send_json_evm_transaction_local_wallet() {
        let private_key = std::env::var("ETHEREUM_PRIVATE_KEY").unwrap();
        let signer: LocalWallet = private_key.try_into().unwrap();
        let from = signer.address().unwrap();
        let to = from.clone();
        // Use a very small amount - 0.0000001 ETH
        let value =
            ethers::utils::parse_ether("0.0000001").unwrap().to_string();
        let tx = serde_json::json!({
            "from": from,
            "to": to,
            "value": value,
        });
        tracing::debug!(
            "transaction: {}",
            serde_json::to_string_pretty(&tx).unwrap()
        );

        let result =
            signer.sign_and_send_json_evm_transaction(tx, None).await;
        tracing::debug!("Transaction result: {:?}", result);

        // Test should pass if transaction succeeds OR if it fails due to insufficient funds
        match result {
            Ok(_) => tracing::debug!("Transaction successful"),
            Err(e) => {
                let err_str = e.to_string().to_lowercase();
                if !err_str.contains("insufficient funds") {
                    panic!("Unexpected error: {}", e);
                }
                tracing::debug!("Test skipped due to insufficient funds");
            }
        }
    }
}
