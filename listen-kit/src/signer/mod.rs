#[cfg(feature = "evm")]
pub mod evm;
#[cfg(feature = "http")]
pub mod privy;
#[cfg(feature = "solana")]
pub mod solana;

use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "evm")]
use self::evm::LocalEvmSigner;
#[cfg(feature = "http")]
use self::privy::PrivySigner;
#[cfg(feature = "solana")]
use self::solana::LocalSolanaSigner;

pub enum Transaction {
    #[cfg(feature = "solana")]
    Solana(solana_sdk::transaction::Transaction),
    #[cfg(feature = "evm")]
    Evm(),
}

pub enum SignerType {
    #[cfg(feature = "solana")]
    LocalSolana(LocalSolanaSigner),
    #[cfg(feature = "evm")]
    LocalEvm(LocalEvmSigner),
    #[cfg(feature = "http")]
    Privy(PrivySigner),
}

#[async_trait]
pub trait TransactionSigner: Send + Sync {
    fn address(&self) -> String {
        unimplemented!()
    }

    fn pubkey(&self) -> String {
        unimplemented!()
    }

    #[cfg(feature = "solana")]
    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String> {
        Err(anyhow::anyhow!(
            "Solana transactions not supported by this signer"
        ))
    }

    #[cfg(feature = "evm")]
    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String> {
        Err(anyhow::anyhow!(
            "EVM transactions not supported by this signer"
        ))
    }

    async fn sign_and_send_encoded_solana_transaction(
        &self,
        _tx: String,
    ) -> Result<String> {
        Err(anyhow::anyhow!(
            "Solana transactions not supported by this signer"
        ))
    }

    async fn sign_and_send_json_evm_transaction(
        &self,
        _tx: serde_json::Value,
    ) -> Result<String> {
        Err(anyhow::anyhow!(
            "EVM transactions not supported by this signer"
        ))
    }
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
