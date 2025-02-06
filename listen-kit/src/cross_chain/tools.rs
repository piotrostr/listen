use crate::common::wrap_unsafe;
use crate::signer::SignerContext;
use anyhow::{anyhow, Result};

use super::lifi::LiFi;
use rig_tool_macro::tool;

#[tool(description = "
Get a quote for a multichain swap (or bridge).

This might be required in case the user wonders how much it would cost to
perform a swap or bridge. It is also good in case you would like to validate the
token addresses and other params with the user before executing

from_token_symbol is the symbol of the token to swap from.
to_token_symbol is the symbol of the token to swap to.
amount is the amount of tokens to swap.

the from_token_symbol and to_token_symbol can either be a solana public key, evm
address or a symbol.

The amount has to be a string to avoid precision loss. The amount is accounting
for decimals, e.g. 1e6 for 1 USDC but 1e18 for 1 SOL.

Supported from_chains:
- sol
- arb

Supported to_chains:
- sol
- arb
")]
pub async fn get_multichain_quote(
    from_token_symbol: String,
    to_token_symbol: String,
    amount: String,
    from_chain: String,
    to_chain: String,
) -> Result<serde_json::Value> {
    let signer = SignerContext::current().await;
    let lifi = LiFi::new(None);

    let from_address = if from_chain == "sol" {
        signer.pubkey()
    } else {
        signer.address()
    };

    let to_address = if to_chain == "sol" {
        signer.pubkey()
    } else {
        signer.address()
    };

    let quote = lifi
        .get_quote(
            &from_chain,
            &to_chain,
            &from_token_symbol,
            &to_token_symbol,
            &from_address,
            &to_address,
            &amount,
        )
        .await
        .map_err(|e| {
            anyhow!(
                "{:#?}",
                e.to_string().chars().take(300).collect::<String>()
            )
        })?;

    Ok(quote.summary())
}

#[tool(description = "
Multichain swap (or bridge).

Use this in case of the user trying to swap tokens that exist on two remote
chains, or would like to bridge the tokens

Don't use this in case you are not certain about all of the params, use the
get_multichain_quote tool instead to validate the params in that case.

from_token_symbol is the symbol of the token to bridge from.
to_token_symbol is the symbol of the token to bridge to.
amount is the amount of tokens to bridge.

The amount has to be a string to avoid precision loss. The amount is accounting
for decimals, e.g. 1e6 for 1 USDC but 1e18 for 1 SOL.

Supported from_chains:
- sol
- arb

Supported to_chains:
- sol
- arb
")]
pub async fn multichain_swap(
    from_token_symbol: String,
    to_token_symbol: String,
    amount: String,
    from_chain: String,
    to_chain: String,
) -> Result<String> {
    let signer = SignerContext::current().await;
    let lifi = LiFi::new(None);

    let from_address = if from_chain == "sol" {
        signer.pubkey()
    } else {
        signer.address()
    };

    let to_address = if to_chain == "sol" {
        signer.pubkey()
    } else {
        signer.address()
    };

    let quote = lifi
        .get_quote(
            &from_chain,
            &to_chain,
            &from_token_symbol,
            &to_token_symbol,
            &from_address,
            &to_address,
            &amount,
        )
        .await
        .map_err(|e| {
            anyhow!(
                "{:#?}",
                e.to_string().chars().take(300).collect::<String>()
            )
        })?;

    match quote.transaction_request {
        Some(transaction_request) => wrap_unsafe(move || async move {
            signer
                .sign_and_send_encoded_solana_transaction(
                    transaction_request.data,
                )
                .await
        })
        .await
        .map_err(|e| anyhow!("{:#?}", e)),
        None => Err(anyhow!("No transaction request")),
    }
}
