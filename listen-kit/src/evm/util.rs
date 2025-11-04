use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use crate::signer::privy::PrivySigner;
use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::{Client, Http};
use anyhow::{anyhow, Result};
use privy::auth::UserSession;
use privy::config::PrivyConfig;
use privy::Privy;

use crate::common::wrap_unsafe;
use crate::ensure_evm_wallet_created;
use crate::signer::evm::LocalEvmSigner;
use crate::signer::SignerContext;

pub type EvmProvider = RootProvider<Http<Client>>;

pub fn chain_id_to_rpc_url(chain_id: u64) -> String {
    let alchemy_api_key = env("ALCHEMY_API_KEY");
    match chain_id {
        1 => format!(
            "https://eth-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        ),
        56 => format!(
            "https://bnb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        ),
        8453 => format!(
            "https://base-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        ),
        42161 => format!(
            "https://arb-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        ),
        480 => format!(
            "https://worldchain-mainnet.g.alchemy.com/v2/{}",
            alchemy_api_key
        ),
        _ => panic!("Unsupported chain ID: {}", chain_id),
    }
}

pub fn make_provider(chain_id: u64) -> Result<EvmProvider> {
    let rpc_url = chain_id_to_rpc_url(chain_id);
    Ok(ProviderBuilder::new().on_http(rpc_url.parse()?))
}

pub fn make_ethers_provider(
    chain_id: u64,
) -> Result<ethers::providers::Provider<ethers::providers::Http>> {
    let rpc_url = chain_id_to_rpc_url(chain_id);
    ethers::providers::Provider::<ethers::providers::Http>::try_from(rpc_url)
        .map_err(|e| anyhow::anyhow!(e))
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

pub const TEST_WALLET_ID: &str = "k0pq0k5an1fvo35m5gm3wn8d";
pub const TEST_ADDRESS: &str = "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770";

pub async fn with_privy_evm_signer_test<Fut, T>(future: Fut) -> Result<T>
where
    Fut: Future<Output = Result<T>> + Send,
{
    SignerContext::with_signer(
        Arc::new(PrivySigner::new(
            Arc::new(Privy::new(PrivyConfig::from_env()?)),
            UserSession {
                wallet_address: Some(TEST_ADDRESS.to_string()),
                evm_wallet_id: Some(TEST_WALLET_ID.to_string()),
                ..Default::default()
            },
            "en".to_string(),
        )),
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
    ensure_evm_wallet_created(signer.clone()).await?;
    let owner = Address::from_str(&signer.address().unwrap())?;

    let tx = wrap_unsafe(move || async move { tx_creator(owner).await })
        .await
        .map_err(|e| anyhow!("{:#?}", e))?;

    wrap_unsafe(move || async move {
        signer.sign_and_send_evm_transaction(tx).await
    })
    .await
    .map_err(|e| anyhow!("{:#?}", e))
}
