use std::str::FromStr;

use alloy::primitives::Address;
use alloy::{
    network::TransactionBuilder, providers::Provider,
    rpc::types::TransactionRequest, signers::local::PrivateKeySigner,
};
use anyhow::{Context, Result};
use uniswap_sdk_core::{prelude::*, token};
use uniswap_v3_sdk::prelude::*;

use super::abi::IERC20;
use super::{transaction::send_transaction, util::EvmProvider};

pub async fn check_allowance(
    token_address: Address,
    owner: Address,
    spender: Address,
    amount: U256,
    provider: &EvmProvider,
) -> Result<bool> {
    let current_allowance = IERC20::new(token_address, provider)
        .allowance(owner, spender)
        .call()
        .await?
        ._0;
    tracing::info!(?current_allowance, ?amount, "Allowance check");

    Ok(current_allowance == U256::MAX)
}

pub async fn trade(
    input_token_address: String,
    input_amount: String,
    output_token_address: String,
    provider: &EvmProvider,
    signer: PrivateKeySigner,
) -> Result<String> {
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

    let amount = U256::from_str(&input_amount)?;

    let gas_price = provider
        .get_gas_price()
        .await
        .context("Failed to get gas price")?;

    if !check_allowance(
        input_addr,
        signer.address(),
        router_address,
        amount,
        provider,
    )
    .await
    .context("Failed to check allowance")?
    {
        tracing::info!(
            ?input_addr,
            ?router_address,
            ?amount,
            "Approving token"
        );
        let call = IERC20::approveCall {
            spender: router_address,
            amount: U256::MAX,
        };

        let request = TransactionRequest::default()
            .with_from(signer.address())
            .with_to(input_addr)
            .with_call(&call)
            .with_gas_price(gas_price);

        send_transaction(request, provider, signer.clone()).await?;
        // should probably wait for the tx here and verify approvals, but retries will handle this
    }

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
            recipient: signer.address(),
            ..Default::default()
        },
    )
    .context("Failed to get swap parameters")?;

    let request = TransactionRequest::default()
        .with_from(signer.address())
        .with_to(router_address)
        .with_input(params.calldata)
        .with_value(params.value)
        .with_gas_price(gas_price);

    send_transaction(request, provider, signer).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evm::util::{make_provider, make_signer};

    #[tokio::test]
    async fn test_trade_evm() {
        let provider = make_provider().unwrap();
        let signer = make_signer().unwrap();

        //  WETH on arbitrum
        let output_token =
            "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1".to_string();
        // USDC on arbitrum
        let input_token =
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string();
        // 1 usdc
        let input_amount = "1000000".to_string();

        let result =
            trade(input_token, input_amount, output_token, &provider, signer)
                .await;

        assert!(result.is_ok(), "Trade failed: {:?}", result);
    }
}
