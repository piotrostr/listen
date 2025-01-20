use crate::jup::Jupiter;
use anyhow::{anyhow, Result};
use solana_sdk::signature::Keypair;

pub async fn trade(
    input_mint: String,
    input_amount: u64,
    output_mint: String,
    slippage_bps: u16,
    keypair: &Keypair,
) -> Result<String> {
    let quote = Jupiter::fetch_quote(
        &input_mint,
        &output_mint,
        input_amount,
        slippage_bps,
    )
    .await
    .map_err(|e| anyhow!("Failed to fetch quote: {}", e.to_string()))?;

    let result = Jupiter::swap(quote, keypair)
        .await
        .map_err(|e| anyhow!("Failed to swap: {}", e.to_string()))?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::constants;
    use crate::util::load_keypair_for_tests;

    use super::*;
    use solana_sdk::native_token::sol_to_lamports;

    #[tokio::test]
    async fn test_trade() {
        let keypair = load_keypair_for_tests();
        let result = trade(
            constants::WSOL.to_string(),
            sol_to_lamports(0.0001),
            "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump".to_string(),
            100,
            &keypair,
        )
        .await;

        assert!(result.is_ok());
    }
}
