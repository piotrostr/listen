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
    use privy::{
        caip2::Caip2, config::PrivyConfig, types::EvmTransaction, Privy,
    };

    use super::*;
    use crate::{
        evm::util::{
            execute_evm_transaction, make_provider, with_local_evm_signer,
            with_privy_evm_signer_test, TEST_WALLET_ID,
        },
        signer::SignerContext,
    };

    #[tokio::test]
    async fn test_transfer_eth() {
        with_local_evm_signer(execute_evm_transaction(
            move |owner| async move {
                create_transfer_eth_tx(
                    owner.to_string(),
                    "10000000000000".to_string(),
                    &make_provider(42161)?,
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
                    &make_provider(42161)?,
                    owner,
                )
                .await
            },
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_transfer_erc20_with_privy_signer() {
        with_privy_evm_signer_test(execute_evm_transaction(
            move |owner: Address| async move {
                let tx = create_transfer_erc20_tx(
                    "0xaf88d065e77c8cc2239327c5edb3a432268e5831".to_string(),
                    owner.to_string(),
                    "1000000".to_string(),
                    &make_provider(42161)?,
                    owner,
                )
                .await?;
                let tried: EvmTransaction = serde_json::from_value(
                    serde_json::to_value(tx.clone())?,
                )?;
                tracing::info!(
                    "tx: {:?}, serialized: {}, tried to privy: {}",
                    tx.clone(),
                    serde_json::to_string_pretty(&tx).unwrap(),
                    serde_json::to_string_pretty(&tried).unwrap()
                );
                Ok(tx)
            },
        ))
        .await
        .expect("Failed to execute evm transaction");
    }

    #[tokio::test]
    async fn test_debug_privy_context() {
        with_privy_evm_signer_test(async move {
            let tx = serde_json::json!({
              "from": "0xccc48877a33a2c14e40c82da843cf4c607abf770",
              "to": "0xaf88d065e77c8cc2239327c5edb3a432268e5831",
              "data": "0xa9059cbb000000000000000000000000ccc48877a33a2c14e40c82da843cf4c607abf77000000000000000000000000000000000000000000000000000000000000f4240",
              "gas_price": "0x121b410"
            });
            let signer = SignerContext::current().await;
            let res = signer.sign_and_send_json_evm_transaction(tx, Some(Caip2::ARBITRUM.to_string())).await?;

            println!("res: {:?}", res);

            Ok(())
    }).await.unwrap()
    }
}
