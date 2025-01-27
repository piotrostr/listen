use anyhow::{anyhow, Result};
use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;

use crate::solana::blockhash::BLOCKHASH_CACHE;
use crate::wallet_manager::{UserSession, WalletManager};
use std::str::FromStr;
use std::sync::Arc;

use super::TransactionSigner;

pub struct PrivySigner {
    wallet_manager: Arc<WalletManager>,
    session: UserSession,
}

impl PrivySigner {
    pub fn new(
        wallet_manager: Arc<WalletManager>,
        session: UserSession,
    ) -> Self {
        Self {
            wallet_manager,
            session,
        }
    }
}

#[async_trait]
impl TransactionSigner for PrivySigner {
    fn pubkey(&self) -> Result<Pubkey> {
        Pubkey::from_str(&self.session.wallet_address)
            .map_err(|e| anyhow!("Invalid wallet address: {}", e))
    }

    async fn sign_and_send_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String> {
        tx.message.recent_blockhash = BLOCKHASH_CACHE.get_blockhash().await?;
        let tx_hash = self
            .wallet_manager
            .sign_and_send_transaction(
                self.session.wallet_address.clone(),
                tx,
            )
            .await?;
        Ok(tx_hash)
    }
}
