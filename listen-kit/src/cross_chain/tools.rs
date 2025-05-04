use std::str::FromStr;
use std::sync::Arc;

use alloy::primitives::FixedBytes;
use alloy::providers::PendingTransactionConfig;
use alloy::providers::Provider;
use anyhow::{anyhow, Result};
use blockhash_cache::{inject_blockhash_into_encoded_tx, BLOCKHASH_CACHE};
use rig_tool_macro::tool;

use crate::common::wrap_unsafe;
use crate::ensure_evm_wallet_created;
use crate::ensure_solana_wallet_created;
use crate::evm::util::make_provider;
use crate::signer::SignerContext;
use crate::signer::TransactionSigner;

use lifi::LiFi;

// TODO!! support sponsored transactions here
// it would save a lot of gas if we could drip on any chain,
// fees are substantially higher if the user has an empty wallet on the dest chain

#[tool(description = "
Get a quote for a multichain swap (or bridge).

This might be required in case the user wonders how much it would cost to
perform a swap or bridge. It is also good in case you would like to validate the
token addresses and other params with the user before executing

The from_token_address and to_token_address can either be a solana public key or EVM address.

The amount has to be a string to avoid precision loss. The amount is accounting
for decimals, e.g. 1e6 for 1 USDC but 1e18 for 1 SOL.

Note that sometimes the quote will return a transaction request, with an address that might require approval.
In that case, you can use the approve_token tool to approve the token.

Supported from_chain values:
- solana: \"1151111081099710\"
- mainnet: \"1\"
- arbitrum: \"42161\"
- base: \"8453\"
- bsc: \"56\"

Supported to_chain values:
- solana: \"1151111081099710\"
- mainnet: \"1\"
- arbitrum: \"42161\"
- base: \"8453\"
- bsc: \"56\"

Special Case: from_token_address/to_token_address for ethereum (any chain) is just \"ETH\"
Solana Address: \"So11111111111111111111111111111111111111112\"

Example:
{{
  \"from_token_address\": \"So11111111111111111111111111111111111111112\", // solana
  \"to_token_address\": \"ETH\", // ethereum
  \"amount\": \"1000000000\", // 1 SOL, 1e9
  \"from_chain\": \"1151111081099710\", // solana
  \"to_chain\": \"1\" // ethereum
}}

if a user hits you with a chain you cannot support, let them know
")]
pub async fn get_quote(
    from_token_address: String,
    to_token_address: String,
    amount: String,
    from_chain: String,
    to_chain: String,
) -> Result<serde_json::Value> {
    let signer = SignerContext::current().await;
    ensure_solana_wallet_created(signer.clone()).await?;
    ensure_evm_wallet_created(signer.clone()).await?;

    let from_chain = clean_quotes(&from_chain);
    let to_chain = clean_quotes(&to_chain);
    let from_token_address = clean_quotes(&from_token_address);
    let to_token_address = clean_quotes(&to_token_address);
    let amount = clean_quotes(&amount);

    #[cfg(feature = "solana")]
    if from_chain == "1151111081099710" && to_chain == "1151111081099710" {
        let quote = crate::solana::jup::Jupiter::fetch_quote(
            &from_token_address,
            &to_token_address,
            amount.parse::<u64>().map_err(|e| anyhow!(e))?,
        )
        .await?;
        return Ok(serde_json::to_value(quote)?);
    }
    let lifi_api_key: Option<String> = match std::env::var("LIFI_API_KEY") {
        Ok(val) => Some(val),
        Err(_) => None,
    };
    let lifi = LiFi::new(lifi_api_key, Some("listen".to_string()));

    let from_address = if from_chain == "1151111081099710"
        || from_chain.to_lowercase() == "sol"
    {
        signer.pubkey()
    } else {
        signer.address()
    };

    let to_address = if to_chain == "1151111081099710"
        || to_chain.to_lowercase() == "sol"
    {
        signer.pubkey()
    } else {
        signer.address()
    };

    let quote = lifi
        .get_quote(
            &from_chain,
            &to_chain,
            &from_token_address,
            &to_token_address,
            &from_address.unwrap(),
            &to_address.unwrap(),
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
This tool can be used to swap tokens on any chain, solana to solana, evm to evm,
solana to evm, evm to solana, etc.

It will automatically pick the best routing for the swap, as long as the chain
parameters and the token addresses are correct and handle approval if needed.

Don't use this in case you are not certain about all of the params, use the
get_multichain_quote tool instead to validate the params in that case.

The from_token_address and to_token_address can either be a solana public key, evm
address or a symbol, try to prioritize the address over the symbol

The amount has to be a string to avoid precision loss. The amount is accounting
for decimals, e.g. 1e6 for 1 USDC but 1e18 for 1 SOL.

Supported from_chain values:
- solana: \"1151111081099710\"
- arbitrum: \"42161\"
- mainnet: \"1\"
- base: \"8453\"
- bsc: \"56\"

Supported to_chain values:
- solana: \"1151111081099710\"
- arbitrum: \"42161\"
- mainnet: \"1\"
- base: \"8453\"
- bsc: \"56\"

Example:
{{
  \"from_token_address\": \"So11111111111111111111111111111111111111112\", // solana
  \"to_token_address\": \"ETH\", // ethereum
  \"amount\": \"1000000000\", // 1 SOL, 1e9
  \"from_chain\": \"1151111081099710\", // solana
  \"to_chain\": \"1\" // ethereum
}}

Special Case: from_token_address/to_token_address for ethereum (any chain) is just \"ETH\"
Solana Address: \"So11111111111111111111111111111111111111112\"

if a user hits you with a chain you cannot support, let them know
")]
pub async fn swap(
    from_token_address: String,
    to_token_address: String,
    amount: String,
    from_chain: String,
    to_chain: String,
) -> Result<String> {
    let signer = SignerContext::current().await;
    ensure_solana_wallet_created(signer.clone()).await?;
    ensure_evm_wallet_created(signer.clone()).await?;

    let from_chain = clean_quotes(&from_chain);
    let to_chain = clean_quotes(&to_chain);
    let from_token_address = clean_quotes(&from_token_address);
    let to_token_address = clean_quotes(&to_token_address);
    let amount = clean_quotes(&amount);

    #[cfg(feature = "solana")]
    if from_chain == "1151111081099710" && to_chain == "1151111081099710" {
        return crate::solana::tools::swap(
            from_token_address,
            amount,
            to_token_address,
        )
        .await;
    }
    let lifi_api_key: Option<String> = match std::env::var("LIFI_API_KEY") {
        Ok(val) => Some(val),
        Err(_) => None,
    };
    let lifi = LiFi::new(lifi_api_key, Some("listen".to_string()));

    let from_address = if from_chain == "1151111081099710"
        || from_chain.to_lowercase() == "sol"
    {
        signer.pubkey()
    } else {
        signer.address()
    };

    let to_address = if to_chain == "1151111081099710"
        || to_chain.to_lowercase() == "sol"
    {
        signer.pubkey()
    } else {
        signer.address()
    };

    let quote = lifi
        .get_quote(
            &from_chain,
            &to_chain,
            &from_token_address,
            &to_token_address,
            &from_address.unwrap(),
            &to_address.unwrap(),
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
        Some(transaction_request) => {
            wrap_unsafe(move || async move {
                if transaction_request.is_solana() {
                    let latest_blockhash =
                        BLOCKHASH_CACHE.get_blockhash().await?.to_string();
                    let encoded_tx = inject_blockhash_into_encoded_tx(
                        &transaction_request.data,
                        &latest_blockhash,
                    )?;
                    signer
                        .sign_and_send_encoded_solana_transaction(encoded_tx)
                        .await
                } else {
                    ensure_lifi_router_approvals(
                        signer.clone(),
                        from_token_address,
                        amount,
                        from_chain.parse::<u64>().map_err(|e| anyhow!(e))?,
                    )
                    .await?;
                    signer
                        .sign_and_send_json_evm_transaction(
                            transaction_request.to_json_rpc()?,
                            None,
                        )
                        .await
                }
            })
            .await
        }
        None => Err(anyhow!("No transaction request")),
    }
}

#[tool(description = "
Check if a token has enough approval for a spender.

token_address is the ERC20 token contract address
spender_address is the address that needs approval
amount is the amount to check approval for (in token decimals)

Returns 'true' if approved, 'false' if not approved
")]
pub async fn check_approval(
    token_address: String,
    spender_address: String,
    amount: String,
    from_chain_caip2: String,
) -> Result<String> {
    let signer = SignerContext::current().await;
    ensure_evm_wallet_created(signer.clone()).await?;

    let owner_address = signer.address();

    let allowance = evm_approvals::get_allowance(
        &token_address,
        &owner_address.unwrap(),
        &spender_address,
        evm_approvals::caip2_to_chain_id(&from_chain_caip2)?,
    )
    .await?;
    let amount = amount
        .parse::<u128>()
        .map_err(|_| anyhow!("Invalid amount"))?;

    Ok((allowance >= amount).to_string())
}

#[tool(description = "
Approve a token for a spender.

token_address is the ERC20 token contract address
spender_address is the address that needs approval
amount is the amount to approve (in token decimals)
")]
pub async fn approve_token(
    token_address: String,
    spender_address: String,
    from_chain_caip2: String,
) -> Result<String> {
    let signer = SignerContext::current().await;
    ensure_evm_wallet_created(signer.clone()).await?;
    let owner_address = signer.address();

    let transaction = evm_approvals::create_approval_transaction(
        &token_address,
        &spender_address,
        &owner_address.unwrap(),
        evm_approvals::caip2_to_chain_id(&from_chain_caip2)?,
    )
    .await?;

    wrap_unsafe(move || async move {
        signer
            .sign_and_send_json_evm_transaction(transaction, None)
            .await
            .map_err(|e| anyhow!(e.to_string()))
    })
    .await?;

    Ok("Approved".to_string())
}

pub const LIFI_DIAMOND_ADDRESS: &str =
    "0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE";

pub async fn ensure_lifi_router_approvals(
    signer: Arc<dyn TransactionSigner>,
    token_address: String,
    amount: String,
    chain_id: u64,
) -> Result<()> {
    if signer.address().is_none() {
        return Err(anyhow!(
            "No address found, it is required for EVM actions!"
        ));
    }
    let allowance = evm_approvals::get_allowance(
        &token_address,
        &signer.address().unwrap(),
        LIFI_DIAMOND_ADDRESS,
        &chain_id.to_string(),
    )
    .await?;
    if allowance < amount.parse::<u128>().map_err(|e| anyhow!(e))? {
        tracing::info!(
            "Approving Lifi Router for {} of {}",
            amount,
            token_address
        );
        let transaction = evm_approvals::create_approval_transaction(
            &token_address,
            LIFI_DIAMOND_ADDRESS,
            &signer.address().unwrap(),
            &chain_id.to_string(),
        )
        .await?;
        let tx_hash = signer
            .sign_and_send_json_evm_transaction(
                transaction,
                Some(format!("eip155:{}", chain_id)),
            )
            .await
            .map_err(|e| anyhow!(e.to_string()))?;
        let provider = make_provider(chain_id)?;
        provider
            .watch_pending_transaction(PendingTransactionConfig::new(
                FixedBytes::from_str(&tx_hash)?,
            ))
            .await?
            .await?;
    }
    Ok(())
}

/// Removes both escaped and unescaped quotes from a string
fn clean_quotes(s: &str) -> String {
    s.replace("\\\"", "") // first remove escaped quotes
        .trim_matches('"') // then remove surrounding quotes
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::solana::util::make_test_signer;

    pub use super::*;

    #[tokio::test]
    async fn test_quote_tool() {
        let params: serde_json::Value = serde_json::from_str(
            "{\"from_token_address\":\"So11111111111111111111111111111111111111112\",\"amount\":\"100000000\",\"from_chain\":\"1151111081099710\",\"to_token_address\":\"0x14feE680690900BA0ccCfC76AD70Fd1b95D10e16\",\"to_chain\":\"1\"}",
        )
        .unwrap();
        let signer = make_test_signer();
        SignerContext::with_signer(signer, async {
            let quote = get_quote(
                params["from_token_address"].to_string(),
                params["to_token_address"].to_string(),
                params["amount"].to_string(),
                params["from_chain"].to_string(),
                params["to_chain"].to_string(),
            )
            .await
            .unwrap();
            println!("quote: {:?}", quote);
            Ok(())
        })
        .await
        .unwrap();
    }
    #[test]
    fn test_clean_quotes() {
        let s = "\"some_param\"";
        let cleaned = clean_quotes(s);
        assert_eq!(cleaned, "some_param");
    }

    #[test]
    fn test_parse_u64() {
        let s = "\"10000\"";
        let cleaned = clean_quotes(s);
        let parsed = cleaned.parse::<u64>();
        assert!(parsed.is_ok(), "{:?}", parsed);
        assert_eq!(parsed.unwrap(), 10000);
    }
}
