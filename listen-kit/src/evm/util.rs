use std::future::Future;
use std::str::FromStr;
use tokio::sync::mpsc;

use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::{Client, Http};
use anyhow::{anyhow, Result};

pub type EvmProvider = RootProvider<Http<Client>>;

pub fn make_provider() -> Result<EvmProvider> {
    let rpc_url = env("ETHEREUM_RPC_URL");
    Ok(ProviderBuilder::new().on_http(rpc_url.parse()?))
}

pub fn make_signer() -> Result<PrivateKeySigner> {
    Ok(PrivateKeySigner::from_str(&env("ETHEREUM_PRIVATE_KEY"))?)
}

pub fn env(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| panic!("{} env var not set", var))
}

pub async fn wrap_unsafe<F, Fut, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let (tx, mut rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let result = f().await;
        let _ = tx.send(result).await;
    });

    rx.recv().await.ok_or_else(|| anyhow!("Channel closed"))?
}
