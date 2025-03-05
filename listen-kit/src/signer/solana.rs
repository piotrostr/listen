use anyhow::Result;
use async_trait::async_trait;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::sync::Arc;

use crate::solana::transaction::send_tx;
use blockhash_cache::BLOCKHASH_CACHE;

use super::TransactionSigner;

pub struct LocalSolanaSigner {
    keypair: Arc<Keypair>,
}

impl LocalSolanaSigner {
    pub fn new(private_key: String) -> Self {
        let keypair = Keypair::from_base58_string(&private_key);
        Self {
            keypair: Arc::new(keypair),
        }
    }
}

#[async_trait]
impl TransactionSigner for LocalSolanaSigner {
    #[cfg(feature = "evm")]
    fn address(&self) -> Option<String> {
        None
    }

    #[cfg(feature = "solana")]
    fn pubkey(&self) -> Option<String> {
        Some(self.keypair.pubkey().to_string())
    }

    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut solana_sdk::transaction::VersionedTransaction,
    ) -> Result<String> {
        let recent_blockhash = BLOCKHASH_CACHE.get_blockhash().await?;
        let mut message = tx.message.clone();
        message.set_recent_blockhash(recent_blockhash);

        // Get the message data and sign it directly with the keypair
        let message_bytes = message.serialize();
        let signature = self.keypair.sign_message(&message_bytes);

        // Update transaction with the new message and signature
        tx.message = message;
        tx.signatures = vec![signature];

        send_tx(tx).await
    }
}
