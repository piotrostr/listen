use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "solana")]
use crate::solana::blockhash::BLOCKHASH_CACHE;
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
pub fn transaction_to_base64(
    transaction: &solana_sdk::transaction::Transaction,
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
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String> {
        use privy::caip2::Caip2;

        tx.message.recent_blockhash = BLOCKHASH_CACHE.get_blockhash().await?;

        self.privy
            .execute_solana_transaction(
                self.pubkey(),
                transaction_to_base64(tx)?,
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
        self.privy
            .execute_evm_transaction(
                self.address(),
                serde_json::to_value(tx)?,
                Caip2::ARBITRUM.to_string(), // TODO paramterize
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
        self.privy
            .execute_evm_transaction(
                self.address(),
                tx,
                Caip2::ARBITRUM.to_string(), // TODO paramterize
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to sign and send json evm transaction: {}",
                    e
                )
            })
    }
}
