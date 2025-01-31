use std::str::FromStr;

use alloy::primitives::Address;
use alloy::{
    network::TransactionBuilder, providers::Provider,
    rpc::types::TransactionRequest,
};
use anyhow::{anyhow, Context, Result};
use uniswap_sdk_core::{prelude::*, token};
use uniswap_v3_sdk::prelude::*;

use super::abi::IERC20;
use super::util::EvmProvider;

pub async fn check_allowance(
    token_address: Address,
    owner: Address,
    spender: Address,
    provider: &EvmProvider,
) -> Result<bool> {
    let current_allowance = IERC20::new(token_address, provider)
        .allowance(owner, spender)
        .call()
        .await?
        ._0;
    tracing::info!(?current_allowance, "Allowance check");

    Ok(current_allowance >= U256::from(u128::MAX))
}

pub async fn create_approve_tx(
    input_token_address: String,
    spender: String,
    owner: String,
    provider: &EvmProvider,
) -> Result<TransactionRequest> {
    // TODO good example for reasoning loop demo
    tracing::info!(?input_token_address, ?spender, "Approving token");
    let input_addr = Address::from_str(&input_token_address)?;
    let spender_addr = Address::from_str(&spender)?;
    let owner_addr = Address::from_str(&owner)?;
    let call = IERC20::approveCall {
        spender: spender_addr,
        amount: U256::MAX,
    };

    // TODO move gas price to global cache
    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    let tx = TransactionRequest::default()
        .with_from(owner_addr)
        .with_to(input_addr)
        .with_call(&call)
        .with_gas_price(gas_price);

    Ok(tx)
    // send_transaction(tx, provider, wallet).await?;
    // should probably wait for the tx here and verify approvals, but retries will handle this
}

pub async fn create_trade_tx(
    input_token_address: String,
    input_amount: String,
    output_token_address: String,
    provider: &EvmProvider,
    owner: Address,
) -> Result<TransactionRequest> {
    // Convert addresses from string to Address type
    let input_addr = Address::from_str(&input_token_address)?;
    let output_addr = Address::from_str(&output_token_address)?;

    // Create token instances
    let chain_id = provider.get_chain_id().await?;
    let input_token = token!(chain_id, input_addr, 18);
    let output_token = token!(chain_id, output_addr, 18);

    // Parse input amount
    let amount_in = CurrencyAmount::from_raw_amount(
        input_token.clone(),
        BigInt::from_str(&input_amount)?,
    )
    .context("Failed to create CurrencyAmount")?;

    let router_address = *SWAP_ROUTER_02_ADDRESSES
        .get(&chain_id)
        .expect("Swap router address not found");

    if !check_allowance(input_addr, owner, router_address, provider)
        .await
        .context("Failed to check allowance")?
    {
        return Err(anyhow!("Allowance not set"));
    }

    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    // Create pool instance
    let pool = Pool::<EphemeralTickMapDataProvider>::from_pool_key_with_tick_data_provider(
        chain_id,
        FACTORY_ADDRESS,
        input_addr,
        output_addr,
        FeeAmount::MEDIUM,
        provider.clone(),
        None,
    )
    .await
    .context("Failed to create pool")?;

    let route = Route::new(vec![pool], input_token, output_token);

    let trade =
        Trade::from_route(route.clone(), amount_in, TradeType::ExactInput)
            .context("Failed to create trade")?;

    let params = swap_call_parameters(
        &mut [trade],
        SwapOptions {
            recipient: owner,
            ..Default::default()
        },
    )
    .context("Failed to get swap parameters")?;

    let request = TransactionRequest::default()
        .with_from(owner)
        .with_to(router_address)
        .with_input(params.calldata)
        .with_value(params.value)
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
    async fn test_approval() {
        let provider = make_provider().unwrap();

        let router_address = *SWAP_ROUTER_02_ADDRESSES
            .get(&provider.get_chain_id().await.unwrap())
            .expect("Router address not found");

        let input_token =
            "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1".to_string();

        with_local_evm_signer(execute_evm_transaction(
            move |owner| async move {
                create_approve_tx(
                    input_token,
                    router_address.to_string(),
                    owner.to_string(),
                    &provider,
                )
                .await
            },
        ))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_trade_evm() {
        let provider = make_provider().unwrap();

        //  WETH on arbitrum
        let output_token =
            "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1".to_string();
        // USDC on arbitrum
        let input_token =
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string();
        // 1 usdc
        let input_amount = "1000000".to_string();

        with_local_evm_signer(execute_evm_transaction(
            move |owner| async move {
                create_trade_tx(
                    input_token,
                    input_amount,
                    output_token,
                    &provider,
                    owner,
                )
                .await
            },
        ))
        .await
        .unwrap();
    }
}
