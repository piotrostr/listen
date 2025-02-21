use std::str::FromStr;

use alloy::primitives::utils::parse_ether;
use alloy::primitives::Address;
use alloy::providers::Provider;
use anyhow::{Context, Result};

use rig_tool_macro::tool;
use uniswap_sdk_core::prelude::SWAP_ROUTER_02_ADDRESSES;

use crate::common::wrap_unsafe;
use crate::signer::SignerContext;

use super::balance::{balance, token_balance};
use super::trade::{check_allowance, create_approve_tx, create_trade_tx};
use super::transfer::{create_transfer_erc20_tx, create_transfer_eth_tx};
use super::util::{execute_evm_transaction, make_provider};

#[tool(description = "
Use this function to verify if a given token has swap router allowance

On EVM, before swapping a token, this function has to be called to verify swap would be successful
")]
pub async fn verify_swap_router_has_allowance(
    token_address: String,
) -> Result<bool> {
    let owner = SignerContext::current().await.address();
    wrap_unsafe(move || async move {
        let provider = make_provider()?;
        let router_address = *SWAP_ROUTER_02_ADDRESSES
            .get(&provider.get_chain_id().await?)
            .context("Router address not found")?;

        check_allowance(
            Address::from_str(&token_address)?,
            Address::from_str(&owner)?,
            router_address,
            &provider,
        )
        .await
    })
    .await
}

#[tool(description = "
Use this function to approve a token for swap router spend

If the verify_swap_router_has_allowance tool returns false, or the swap fails with 
allowance error, call this function to approve the token for swap router spend
")]
pub async fn approve_token_for_router_spend(
    input_token_address: String,
) -> Result<String> {
    let provider = make_provider()?;
    let router_address = wrap_unsafe(move || async move {
        let router_address = *SWAP_ROUTER_02_ADDRESSES
            .get(&make_provider()?.get_chain_id().await?)
            .context("Router address not found")?;

        Ok(router_address)
    })
    .await?;

    execute_evm_transaction(move |owner| async move {
        create_approve_tx(
            input_token_address,
            router_address.to_string(),
            owner.to_string(),
            &provider,
        )
        .await
    })
    .await
}

#[tool(description = "
Use this function to swap any tokens on EVM using Uniswap

The function supports tokens that are on the same chain
")]
pub async fn trade(
    input_token_address: String,
    input_amount: String,
    output_token_address: String,
) -> Result<String> {
    let input_amount = if input_amount.contains('.') {
        parse_ether(&input_amount)?.to_string()
    } else {
        input_amount
    };
    execute_evm_transaction(move |owner| async move {
        create_trade_tx(
            input_token_address,
            input_amount,
            output_token_address,
            &make_provider()?,
            owner,
        )
        .await
    })
    .await
}

#[tool(description = "
Transfer ETH to a given address

This function is dangerous, as transfers are irreversible

Before calling this function, the recipient address has to ALWAYS be 
double-checked with the user 
")]
pub async fn transfer_eth(
    recipient: String,
    amount: String,
) -> Result<String> {
    execute_evm_transaction(move |owner| async move {
        create_transfer_eth_tx(recipient, amount, &make_provider()?, owner)
            .await
    })
    .await
}

#[tool(description = "
Transfer ERC20 tokens to a given address

This function is dangerous, as transfers are irreversible

Before calling this function, the recipient address has to ALWAYS be 
double-checked with the user 
")]
pub async fn transfer_erc20(
    recipient: String,
    token_address: String,
    amount: String,
) -> Result<String> {
    execute_evm_transaction(move |owner| async move {
        create_transfer_erc20_tx(
            token_address,
            recipient,
            amount,
            &make_provider()?,
            owner,
        )
        .await
    })
    .await
}

#[tool(description = "
This function returns the ethereum wallet address you are currently using
")]
pub async fn wallet_address() -> Result<String> {
    Ok(SignerContext::current().await.address())
}

#[tool]
pub async fn get_eth_balance(address: String) -> Result<String> {
    wrap_unsafe(
        move || async move { balance(&make_provider()?, address).await },
    )
    .await
}

#[tool]
pub async fn get_erc20_balance(
    token_address: String,
    address: String,
) -> Result<String> {
    wrap_unsafe(move || async move {
        token_balance(address, token_address, &make_provider()?).await
    })
    .await
}
