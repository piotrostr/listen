use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use anyhow::{Context, Result};

use crate::evm::util::EvmProvider;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
    }
}

pub async fn transfer_eth(
    from: Address,
    to: Address,
    amount: U256,
    provider: &EvmProvider,
    signer: PrivateKeySigner,
) -> Result<String> {
    // Get the current gas price
    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    // Create transaction request
    let request = TransactionRequest::default()
        .with_from(from)
        .with_to(to)
        .with_value(amount)
        .with_gas_price(gas_price);

    // Estimate gas
    let gas_limit = provider
        .estimate_gas(&request)
        .await
        .context("Failed to estimate gas")?;

    // Build the transaction with estimated gas
    let tx = request
        .with_gas_limit(gas_limit)
        .with_chain_id(provider.get_chain_id().await?)
        .with_nonce(provider.get_transaction_count(from).await?)
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

pub async fn transfer_erc20(
    provider: &EvmProvider,
    token_address: Address,
    to: Address,
    amount: U256,
) -> Result<String> {
    // Create contract instance
    let contract = IERC20::new(token_address, provider);

    // Get the current gas price
    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    // Build and send the transfer transaction
    let tx_hash = contract
        .transfer(to, amount)
        .gas_price(gas_price)
        .send()
        .await
        .context("Failed to send transaction")?
        .watch()
        .await
        .context("Failed to get transaction receipt")?;

    Ok(tx_hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evm::util::{make_provider, make_signer};
    use alloy::primitives::{address, U256};

    #[tokio::test]
    async fn test_transfer_eth() {
        let provider = make_provider().unwrap();
        let signer = make_signer().unwrap();
        let from = signer.address();
        let to = signer.address();
        let amount = U256::from(10000000000000u64); // 0.00001 ETH

        let result = transfer_eth(from, to, amount, &provider, signer).await;
        assert!(result.is_ok(), "Transfer failed: {:?}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_transfer_erc20() {
        let provider = make_provider().unwrap();
        let signer = make_signer().unwrap();
        let to = signer.address();
        // USDC token address on ARB mainnet
        let token = address!("0xaf88d065e77c8cc2239327c5edb3a432268e5831");
        let amount = U256::from(1000000); // 1 USDC (6 decimals)

        let result = transfer_erc20(&provider, token, to, amount).await;
        assert!(result.is_ok(), "Transfer failed: {:?}", result);
    }
}
