use crate::common::wrap_unsafe;
use crate::signer::SignerContext;
use anyhow::{anyhow, Result};

use super::lifi::LiFi;
use rig_tool_macro::tool;

#[tool]
pub async fn bridge_from_sol_to_arb(
    from_token_symbol: String,
    to_token_symbol: String,
    amount: String,
) -> Result<String> {
    let signer = SignerContext::current().await;
    let lifi = LiFi::new(None);
    let from_address = signer.pubkey();
    let to_address = signer.address();

    let quote = lifi
        .get_quote(
            "sol",
            "arb",
            &from_token_symbol,
            &to_token_symbol,
            &from_address,
            &to_address,
            &amount,
        )
        .await?;

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
