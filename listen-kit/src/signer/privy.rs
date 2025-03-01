use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

#[cfg(feature = "solana")]
use blockhash_cache::BLOCKHASH_CACHE;
use privy::{auth::UserSession, caip2::Caip2, util::base64encode, Privy};
use std::sync::Arc;

use super::TransactionSigner;

pub struct PrivySigner {
    privy: Arc<Privy>,
    session: UserSession,
}

impl PrivySigner {
    pub fn new(privy: Arc<Privy>, session: UserSession) -> Self {
        Self { privy, session }
    }
}

#[cfg(feature = "solana")]
pub fn transaction_to_base64<T: Serialize>(
    transaction: &T,
) -> anyhow::Result<String> {
    let serialized = bincode::serialize(transaction)?;
    Ok(base64encode(&serialized))
}

#[async_trait]
impl TransactionSigner for PrivySigner {
    fn address(&self) -> String {
        self.session.wallet_address.clone()
    }

    fn pubkey(&self) -> String {
        self.session.pubkey.clone()
    }

    #[cfg(feature = "solana")]
    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut solana_sdk::transaction::VersionedTransaction,
    ) -> Result<String> {
        tx.message
            .set_recent_blockhash(BLOCKHASH_CACHE.get_blockhash().await?);

        let encoded_tx = transaction_to_base64(tx)?;

        self.privy
            .execute_solana_transaction(
                self.pubkey(),
                encoded_tx,
                Caip2::SOLANA.to_string(),
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to sign and send solana transaction: {}",
                    e
                )
            })
    }

    #[cfg(feature = "evm")]
    async fn sign_and_send_evm_transaction(
        &self,
        tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String> {
        let caip2 =
            tx.chain_id.map_or(Caip2::ARBITRUM.to_string(), |chain_id| {
                Caip2::from_chain_id(chain_id).to_string()
            });
        self.privy
            .execute_evm_transaction(
                self.address(),
                serde_json::to_value(tx)?,
                caip2,
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to sign and send evm transaction: {}",
                    e
                )
            })
    }

    async fn sign_and_send_encoded_solana_transaction(
        &self,
        encoded_transaction: String,
    ) -> Result<String> {
        self.privy
            .execute_solana_transaction(
                self.pubkey(),
                encoded_transaction,
                Caip2::SOLANA.to_string(),
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to sign and send encoded solana transaction: {}",
                    e
                )
            })
    }

    async fn sign_and_send_json_evm_transaction(
        &self,
        tx: serde_json::Value,
    ) -> Result<String> {
        let caip2 = match tx["chain_id"].as_u64() {
            Some(chain_id) => Caip2::from_chain_id(chain_id),
            None => {
                return Err(anyhow::anyhow!(
                    "Chain ID is required for EVM transactions"
                ))
            }
        };
        self.privy
            .execute_evm_transaction(self.address(), tx, caip2.to_string())
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to sign and send json evm transaction: {}",
                    e
                )
            })
    }
}
