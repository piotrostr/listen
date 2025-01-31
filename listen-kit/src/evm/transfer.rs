use std::str::FromStr;

use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use anyhow::{Context, Result};

use super::abi::IERC20;
use super::util::EvmProvider;

pub async fn create_transfer_eth_tx(
    to: String,
    amount: String,
    provider: &EvmProvider,
    owner: Address,
) -> Result<TransactionRequest> {
    // Get the current gas price
    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    // Create transaction request
    let request = TransactionRequest::default()
        .with_from(owner)
        .with_to(Address::from_str(&to)?)
        .with_value(U256::from_str(&amount)?)
        .with_gas_price(gas_price);

    Ok(request)
}

pub async fn create_transfer_erc20_tx(
    token_address: String,
    to: String,
    amount: String,
    provider: &EvmProvider,
    owner: Address,
) -> Result<TransactionRequest> {
    let call = IERC20::transferCall {
        to: Address::from_str(&to)?,
        amount: U256::from_str(&amount)?,
    };

    // Get the current gas price
    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    let request = TransactionRequest::default()
        .with_from(owner)
        .with_to(Address::from_str(&token_address)?)
        .with_call(&call)
        .with_gas_price(gas_price);

    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evm::util::{
        execute_evm_transaction, make_provider, with_local_evm_signer,
    };

    #[tokio::test]
    async fn test_transfer_eth() {
        with_local_evm_signer(execute_evm_transaction(
            move |owner| async move {
                create_transfer_eth_tx(
                    owner.to_string(),
                    "10000000000000".to_string(),
                    &make_provider()?,
                    owner,
                )
                .await
            },
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_transfer_erc20() {
        with_local_evm_signer(execute_evm_transaction(
            move |owner: Address| async move {
                // USDC on ARB
                let token_address =
                    "0xaf88d065e77c8cc2239327c5edb3a432268e5831".to_string();
                create_transfer_erc20_tx(
                    token_address,
                    owner.to_string(),
                    "1000000".to_string(), // 1 USDC
                    &make_provider()?,
                    owner,
                )
                .await
            },
        ))
        .await
        .unwrap();
    }
}
