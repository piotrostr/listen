use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use anyhow::Result;

use crate::evm::util::EvmProvider;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
    }
}

pub async fn transfer_eth(
    provider: &EvmProvider,
    from: Address,
    to: Address,
    amount: U256,
) -> Result<String> {
    let tx = TransactionRequest::default()
        .with_from(from)
        .with_to(to)
        .with_value(amount)
        .with_gas_limit(21_000) // Standard ETH transfer gas limit
        .with_chain_id(provider.get_chain_id().await?);

    // Send transaction and wait for receipt
    let tx_hash = provider.send_transaction(tx).await?.watch().await?;

    Ok(tx_hash.to_string())
}

pub async fn transfer_erc20(
    provider: &EvmProvider,
    token_address: Address,
    to: Address,
    amount: U256,
) -> Result<String> {
    // Create contract instance
    let contract = IERC20::new(token_address, provider);

    // Build and send the transfer transaction
    let tx_hash = contract.transfer(to, amount).send().await?.watch().await?;

    Ok(tx_hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evm::util::{make_provider, make_signer};
    use alloy::primitives::{address, U256};

    #[tokio::test]
    async fn test_transfer_eth() {
        let provider = make_provider().unwrap();
        let signer = make_signer().unwrap();
        let from = signer.address();
        let to = signer.address();
        let amount = U256::from(1000000000000000u64); // 0.001 ETH

        let result = transfer_eth(&provider, from, to, amount).await;
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test]
    async fn test_transfer_erc20() {
        let provider = make_provider().unwrap();
        let signer = make_signer().unwrap();
        let to = signer.address();
        // USDC token address on Ethereum mainnet
        let token = address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        let amount = U256::from(1000000); // 1 USDC (6 decimals)

        let result = transfer_erc20(&provider, token, to, amount).await;
        assert!(result.is_ok(), "{:?}", result);
    }
}
