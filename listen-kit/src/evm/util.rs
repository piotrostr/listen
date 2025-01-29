use std::str::FromStr;

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
