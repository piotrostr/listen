use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::{Client, Http};
use anyhow::{anyhow, Result};

use crate::common::wrap_unsafe;
use crate::signer::evm::LocalEvmSigner;
use crate::signer::SignerContext;

pub type EvmProvider = RootProvider<Http<Client>>;

pub fn make_provider() -> Result<EvmProvider> {
    let rpc_url = env("ETHEREUM_RPC_URL");
    Ok(ProviderBuilder::new().on_http(rpc_url.parse()?))
}

pub fn make_signer() -> Result<PrivateKeySigner> {
    Ok(PrivateKeySigner::from_str(&env("ETHEREUM_PRIVATE_KEY"))?)
}

pub fn make_wallet() -> Result<EthereumWallet> {
    Ok(EthereumWallet::from(make_signer()?))
}

pub fn env(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| panic!("{} env var not set", var))
}

pub async fn with_local_evm_signer<Fut, T>(future: Fut) -> Result<T>
where
    Fut: Future<Output = Result<T>> + Send,
{
    SignerContext::with_signer(
        Arc::new(LocalEvmSigner::new(env("ETHEREUM_PRIVATE_KEY"))),
        future,
    )
    .await
}

pub async fn execute_evm_transaction<F, Fut>(tx_creator: F) -> Result<String>
where
    F: FnOnce(Address) -> Fut + Send + 'static,
    Fut: Future<Output = Result<TransactionRequest>> + Send + 'static,
{
    let signer = SignerContext::current().await;
    let owner = Address::from_str(&signer.address())?;

    let tx = wrap_unsafe(move || async move { tx_creator(owner).await })
        .await
        .map_err(|e| anyhow!("{:#?}", e))?;

    wrap_unsafe(move || async move {
        signer.sign_and_send_evm_transaction(tx).await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}
