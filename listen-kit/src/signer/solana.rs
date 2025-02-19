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
    fn address(&self) -> String {
        unimplemented!()
    }

    #[cfg(feature = "solana")]
    fn pubkey(&self) -> String {
        self.keypair.pubkey().to_string()
    }

    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String> {
        let recent_blockhash = BLOCKHASH_CACHE.get_blockhash().await?;
        tx.try_sign(&[&*self.keypair], recent_blockhash)?;
        send_tx(tx).await
    }
}
