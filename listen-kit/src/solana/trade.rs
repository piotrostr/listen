use crate::solana::jup::Jupiter;
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;

pub async fn create_jupiter_swap_transaction(
    input_mint: String,
    input_amount: u64,
    output_mint: String,
    owner: &Pubkey,
) -> Result<VersionedTransaction> {
    let quote = Jupiter::fetch_quote(&input_mint, &output_mint, input_amount)
        .await
        .map_err(|e| anyhow!("Failed to fetch quote: {}", e.to_string()))?;

    let tx = Jupiter::swap(quote, owner)
        .await
        .map_err(|e| anyhow!("Failed to swap: {}", e.to_string()))?;

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use crate::solana::{constants, util::load_keypair_for_tests};

    use super::*;
    use solana_sdk::native_token::sol_to_lamports;
    use solana_sdk::signer::Signer;

    #[tokio::test]
    async fn test_jupiter_swap() {
        let keypair = load_keypair_for_tests();
        let result = create_jupiter_swap_transaction(
            constants::WSOL.to_string(),
            sol_to_lamports(0.001),
            "FUAfBo2jgks6gB4Z4LfZkqSZgzNucisEHqnNebaRxM1P".to_string(),
            &keypair.pubkey(),
        )
        .await;
        tracing::info!("{:?}", result);

        assert!(result.is_ok());
    }
}
