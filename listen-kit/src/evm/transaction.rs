use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use anyhow::{Context, Result};
use std::time::Duration;
use tokio::time::sleep;

use super::util::EvmProvider;

pub async fn send_transaction(
    request: TransactionRequest,
    provider: &EvmProvider,
    wallet: &EthereumWallet,
) -> Result<String> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY: Duration = Duration::from_secs(1);

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            tracing::info!("Retry attempt {} for transaction", attempt);
            sleep(RETRY_DELAY).await;
        }

        match try_send_transaction(request.clone(), provider, wallet).await {
            Ok(hash) => return Ok(hash),
            Err(e) => {
                if attempt == MAX_RETRIES - 1 {
                    return Err(e);
                }
                tracing::warn!("Transaction failed: {:?}. Retrying...", e);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to send transaction after all retries"
    ))
}

async fn try_send_transaction(
    request: TransactionRequest,
    provider: &EvmProvider,
    wallet: &EthereumWallet,
) -> Result<String> {
    tracing::info!(?request, "Sending transaction");

    let address = wallet.default_signer().address();

    // Get the latest nonce
    let nonce = provider
        .get_transaction_count(address)
        .await
        .context("Failed to get nonce")?;

    // Estimate gas
    let gas_limit = provider
        .estimate_gas(&request)
        .await
        .context("Failed to estimate gas")?;

    // Build the transaction with estimated gas
    let tx = request
        .with_gas_limit(gas_limit)
        .with_chain_id(provider.get_chain_id().await?)
        .with_nonce(nonce)
        .build(wallet)
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
