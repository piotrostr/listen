use anyhow::Result;
use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::sync::Arc;

use crate::solana::blockhash::BLOCKHASH_CACHE;
use crate::solana::transaction::send_tx;

use super::TransactionSigner;

pub struct LocalSigner {
    keypair: Arc<Keypair>,
}

impl LocalSigner {
    pub fn new(private_key: String) -> Self {
        let keypair = Keypair::from_base58_string(&private_key);
        Self {
            keypair: Arc::new(keypair),
        }
    }
}

#[async_trait]
impl TransactionSigner for LocalSigner {
    fn pubkey(&self) -> Result<Pubkey> {
        Ok(self.keypair.pubkey())
    }

    async fn sign_and_send_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String> {
        let recent_blockhash = BLOCKHASH_CACHE.get_blockhash().await?;
        tx.try_sign(&[&*self.keypair], recent_blockhash)?;
        send_tx(tx).await
    }
}
