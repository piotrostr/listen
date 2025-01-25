use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result};

use super::util::EvmProvider;

pub async fn send_transaction(
    request: TransactionRequest,
    provider: &EvmProvider,
    signer: PrivateKeySigner,
) -> Result<String> {
    // Estimate gas
    let gas_limit = provider
        .estimate_gas(&request)
        .await
        .context("Failed to estimate gas")?;

    // Build the transaction with estimated gas
    let tx = request
        .with_gas_limit(gas_limit)
        .with_chain_id(provider.get_chain_id().await?)
        .with_nonce(provider.get_transaction_count(signer.address()).await?)
        .build(&EthereumWallet::from(signer))
        .await?;

    // Send transaction and wait for receipt
    let tx_hash = provider
        .send_tx_envelope(tx)
        .await
        .context("Failed to send transaction")?
        .watch()
        .await
        .context("Failed to get transaction receipt")?;

    Ok(tx_hash.to_string())
}
