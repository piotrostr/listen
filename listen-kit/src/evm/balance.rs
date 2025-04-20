use std::str::FromStr;

use super::abi::IERC20;
use alloy::primitives::Address;
use alloy::providers::Provider;
use anyhow::Result;

use crate::evm::util::EvmProvider;

pub async fn balance(
    provider: &EvmProvider,
    address: String,
) -> Result<String> {
    let balance = provider.get_balance(Address::from_str(&address)?).await?;
    Ok(balance.to_string())
}

pub async fn token_balance(
    owner: String,
    token_address: String,
    provider: &EvmProvider,
) -> Result<(String, u8)> {
    let balance = IERC20::new(Address::from_str(&token_address)?, provider)
        .balanceOf(Address::from_str(&owner)?)
        .call()
        .await?
        ._0;

    let decimals = IERC20::new(Address::from_str(&token_address)?, provider)
        .decimals()
        .call()
        .await?
        ._0;

    Ok((balance.to_string(), decimals))
}

#[cfg(test)]
mod tests {
    use crate::evm::util::{make_provider, make_signer};

    use super::*;

    #[tokio::test]
    async fn test_balance() {
        let provider = make_provider(42161).unwrap();
        let signer = make_signer().unwrap();

        let balance = balance(&provider, signer.address().to_string())
            .await
            .unwrap();
        assert_ne!(balance, "0");
    }

    #[tokio::test]
    async fn test_token_balance() {
        let provider = make_provider(56).unwrap();
        let signer = make_signer().unwrap();

        let balance = token_balance(
            signer.address().to_string(),
            "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82".to_string(),
            &provider,
        )
        .await
        .unwrap();
        assert_eq!(balance, ("2635746907360432808".to_string(), 18u8)); // brittle, will fail if the balance changes
    }
}
