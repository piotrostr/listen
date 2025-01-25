use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;

use crate::evm::util::EvmProvider;

pub async fn balance(
    provider: &EvmProvider,
    signer: &PrivateKeySigner,
) -> Result<String> {
    let balance = provider.get_balance(signer.address()).await?;
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

        let balance = balance(&provider, &signer).await.unwrap();
        assert_ne!(balance, "0");
    }
}
