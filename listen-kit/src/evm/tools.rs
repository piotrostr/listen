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
use crate::ensure_evm_wallet_created;

#[tool(description = "
Use this function to verify if a given token has swap router allowance

On EVM, before swapping a token, this function has to be called to verify swap would be successful
")]
pub async fn verify_swap_router_has_allowance(
    token_address: String,
    chain_id: u64,
) -> Result<bool> {
    let signer = SignerContext::current().await;
    ensure_evm_wallet_created(signer.clone()).await?;
    let owner = signer.address().unwrap();
    wrap_unsafe(move || async move {
        let provider = make_provider(chain_id)?;
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
    chain_id: u64,
) -> Result<String> {
    let provider = make_provider(chain_id)?;
    let router_address = wrap_unsafe(move || async move {
        let router_address = *SWAP_ROUTER_02_ADDRESSES
            .get(&chain_id)
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
    chain_id: u64,
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
            &make_provider(chain_id)?,
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
    chain_id: u64,
) -> Result<String> {
    execute_evm_transaction(move |owner| async move {
        create_transfer_eth_tx(
            recipient,
            amount,
            &make_provider(chain_id)?,
            owner,
        )
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
    chain_id: u64,
) -> Result<String> {
    execute_evm_transaction(move |owner| async move {
        create_transfer_erc20_tx(
            token_address,
            recipient,
            amount,
            &make_provider(chain_id)?,
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
    let signer = SignerContext::current().await;
    ensure_evm_wallet_created(signer.clone()).await?;
    Ok(signer.address().unwrap())
}

#[tool(description = "
For the address, return the balance of the native token and chain ID
Most chains it will return Ethereum, in case of BSC it will return BNB balance (also 18 decimals)
")]
pub async fn get_eth_balance(
    address: String,
    chain_id: u64,
) -> Result<(String, u64)> {
    wrap_unsafe(move || async move {
        let balance = balance(&make_provider(chain_id)?, address).await?;
        Ok((balance, chain_id))
    })
    .await
}

#[tool]
pub async fn get_erc20_balance(
    token_address: String,
    address: String,
    chain_id: u64,
) -> Result<serde_json::Value> {
    wrap_unsafe(move || async move {
        let (balance, decimals) = token_balance(
            address.clone(),
            token_address.clone(),
            &make_provider(chain_id)?,
        )
        .await?;
        Ok(serde_json::json!({
            "balance": balance,
            "decimals": decimals,
            "token_address": token_address,
            "chain_id": chain_id,
        }))
    })
    .await
}
