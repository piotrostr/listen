use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use anyhow::{Context, Result};

use super::abi::IERC20;
use super::transaction::send_transaction;
use super::util::EvmProvider;

pub async fn transfer_eth(
    from: Address,
    to: Address,
    amount: U256,
    provider: &EvmProvider,
    wallet: &EthereumWallet,
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

    send_transaction(request, provider, wallet).await
}

pub async fn transfer_erc20(
    from: Address,
    token_address: Address,
    to: Address,
    amount: U256,
    provider: &EvmProvider,
    wallet: &EthereumWallet,
) -> Result<String> {
    // Create contract instance
    let call = IERC20::transferCall { to, amount };

    // Get the current gas price
    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    let request = TransactionRequest::default()
        .with_from(from)
        .with_to(token_address)
        .with_call(&call)
        .with_gas_price(gas_price);

    send_transaction(request, provider, wallet).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evm::util::{make_provider, make_wallet};
    use alloy::primitives::{address, U256};

    #[tokio::test]
    async fn test_transfer_eth() {
        let provider = make_provider().unwrap();
        let wallet = make_wallet().unwrap();
        let address = wallet.default_signer().address();
        let from = address;
        let to = address;
        let amount = U256::from(10000000000000u64); // 0.00001 ETH

        let result = transfer_eth(from, to, amount, &provider, &wallet).await;
        assert!(result.is_ok(), "Transfer failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_transfer_erc20() {
        let provider = make_provider().unwrap();
        let wallet = make_wallet().unwrap();
        let address = wallet.default_signer().address();
        let from = address;
        let to = address;
        // USDC token address on ARB mainnet
        let token = address!("0xaf88d065e77c8cc2239327c5edb3a432268e5831");
        let amount = U256::from(1000000); // 1 USDC (6 decimals)

        let result =
            transfer_erc20(from, token, to, amount, &provider, &wallet).await;
        assert!(result.is_ok(), "Transfer failed: {:?}", result);
    }
}
