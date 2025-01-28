pub mod local;
#[cfg(feature = "http")]
pub mod privy;

use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;

use self::local::LocalSigner;
#[cfg(feature = "http")]
use self::privy::PrivySigner;

#[async_trait]
pub trait TransactionSigner: Send + Sync {
    fn pubkey(&self) -> Result<Pubkey>;
    async fn sign_and_send_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String>;
}

pub enum SignerType {
    Local(LocalSigner),
    #[cfg(feature = "http")]
    Privy(PrivySigner),
}

tokio::task_local! {
    static CURRENT_SIGNER: Arc<dyn TransactionSigner>;
}

pub struct SignerContext;

impl SignerContext {
    pub async fn with_signer<T>(
        signer: Arc<dyn TransactionSigner>,
        f: impl Future<Output = Result<T>> + Send,
    ) -> Result<T> {
        CURRENT_SIGNER.scope(signer, f).await
    }

    pub async fn current() -> Arc<dyn TransactionSigner> {
        CURRENT_SIGNER.get().clone()
    }
}
