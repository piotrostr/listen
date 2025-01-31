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
) -> Result<String> {
    let balance = IERC20::new(Address::from_str(&token_address)?, provider)
        .balanceOf(Address::from_str(&owner)?)
        .call()
        .await?
        ._0;

    Ok(balance.to_string())
}

#[cfg(test)]
mod tests {
    use crate::evm::util::{make_provider, make_signer};

    use super::*;

    #[tokio::test]
    async fn test_balance() {
        let provider = make_provider().unwrap();
        let signer = make_signer().unwrap();

        let balance = balance(&provider, signer.address().to_string())
            .await
            .unwrap();
        assert_ne!(balance, "0");
    }
}
