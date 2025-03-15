use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};

use privy::caip2::Caip2;

use super::retry::retry_with_backoff;
use crate::jup::Jupiter;
use privy::util::base64encode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOrder {
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub from_chain_caip2: String,
    pub to_chain_caip2: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SwapOrderError {
    #[error("Invalid CAIP2")]
    InvalidCaip2,

    #[error("LiFi error: {0}")]
    LiFiError(lifi::LiFiError),

    #[error("No transaction request")]
    NoTransactionRequest,

    #[error("Serialize error: {0}")]
    SerializeError(anyhow::Error),

    #[error("Jupiter error: {0}")]
    JupiterError(anyhow::Error),

    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(anyhow::Error),

    #[error("Invalid amount: {0}")]
    InvalidAmount(anyhow::Error),

    #[error("EVM wallet not available")]
    EVMWalletNotAvailable,

    #[error("Solana wallet not available")]
    SolanaWalletNotAvailable,

    #[error("No wallet address")]
    NoWalletAddress,
}

pub fn is_solana(caip2: &str) -> bool {
    caip2.starts_with("solana:")
}

pub fn is_evm(caip2: &str) -> bool {
    caip2.starts_with("eip155:")
}

impl SwapOrder {
    pub fn is_evm(&self) -> bool {
        is_evm(&self.from_chain_caip2)
    }

    pub fn is_solana(&self) -> bool {
        is_solana(&self.from_chain_caip2)
    }
}

// Map of CAIP2 identifiers to LiFi chain IDs
static CHAIN_ID_MAP: Lazy<HashMap<&'static str, u64>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Solana special case
    m.insert(Caip2::SOLANA, 1151111081099710);
    // EVM chains
    m.insert(Caip2::ETHEREUM, 1);
    m.insert(Caip2::BSC, 56);
    m.insert(Caip2::ARBITRUM, 42161);
    m.insert(Caip2::BASE, 8453);
    m.insert(Caip2::BLAST, 81457);
    m.insert(Caip2::AVALANCHE, 43114);
    m.insert(Caip2::POLYGON, 137);
    m.insert(Caip2::SCROLL, 534352);
    m.insert(Caip2::OPTIMISM, 10);
    m.insert(Caip2::LINEA, 59144);
    m.insert(Caip2::GNOSIS, 100);
    m.insert(Caip2::FANTOM, 250);
    m.insert(Caip2::MOONRIVER, 1285);
    m.insert(Caip2::MOONBEAM, 1284);
    m.insert(Caip2::BOBA, 288);
    m.insert(Caip2::MODE, 34443);
    m.insert(Caip2::METIS, 1088);
    m.insert(Caip2::LISK, 1135);
    m.insert(Caip2::AURORA, 1313161554);
    m.insert(Caip2::SEI, 1329);
    m.insert(Caip2::IMMUTABLE, 13371);
    m.insert(Caip2::GRAVITY, 1625);
    m.insert(Caip2::TAIKO, 167000);
    m.insert(Caip2::CRONOS, 25);
    m.insert(Caip2::FRAXTAL, 252);
    m.insert(Caip2::ABSTRACT, 2741);
    m.insert(Caip2::CELO, 42220);
    m.insert(Caip2::WORLD, 480);
    m.insert(Caip2::MANTLE, 5000);
    m.insert(Caip2::BERACHAIN, 80094);
    m
});

fn caip2_to_chain_id(caip2: &str) -> Option<u64> {
    CHAIN_ID_MAP.get(caip2).copied()
}

pub enum SwapOrderTransaction {
    Evm(serde_json::Value),
    Solana(String),
}

pub async fn swap_order_to_transaction(
    order: &SwapOrder,
    lifi: &lifi::LiFi,
    wallet_address: Option<String>, // evm output
    pubkey: Option<String>,         // solana output
) -> Result<SwapOrderTransaction, SwapOrderError> {
    let from_chain_id =
        caip2_to_chain_id(&order.from_chain_caip2).ok_or(SwapOrderError::InvalidCaip2)?;
    let to_chain_id =
        caip2_to_chain_id(&order.to_chain_caip2).ok_or(SwapOrderError::InvalidCaip2)?;

    if wallet_address.is_none() && order.is_evm() {
        return Err(SwapOrderError::EVMWalletNotAvailable);
    }
    if pubkey.is_none() && order.is_solana() {
        return Err(SwapOrderError::SolanaWalletNotAvailable);
    }

    if from_chain_id == to_chain_id && is_solana(&order.from_chain_caip2) {
        tracing::info!("Solana swap order to transaction");
        if let Some(pubkey) = pubkey {
            return retry_with_backoff("solana swap to transaction", || async {
                try_solana_swap_order_to_transaction(order, &pubkey).await
            })
            .await;
        } else {
            return Err(SwapOrderError::NoWalletAddress);
        }
    }

    let wallet_address = wallet_address.unwrap();
    let pubkey = pubkey.unwrap();

    retry_with_backoff("lifi swap to transaction", || async {
        try_lifi_swap_order_to_transaction(order, lifi, &wallet_address, &pubkey).await
    })
    .await
}

// Helper function that actually performs the LiFi swap operation
async fn try_lifi_swap_order_to_transaction(
    order: &SwapOrder,
    lifi: &lifi::LiFi,
    wallet_address: &str,
    pubkey: &str,
) -> Result<SwapOrderTransaction, SwapOrderError> {
    let from_chain_id =
        caip2_to_chain_id(&order.from_chain_caip2).ok_or(SwapOrderError::InvalidCaip2)?;
    let to_chain_id =
        caip2_to_chain_id(&order.to_chain_caip2).ok_or(SwapOrderError::InvalidCaip2)?;

    let from_address = if is_evm(&order.from_chain_caip2) {
        wallet_address
    } else {
        pubkey
    };

    let to_address = if is_evm(&order.to_chain_caip2) {
        wallet_address
    } else {
        pubkey
    };

    let quote = lifi
        .get_quote(
            &from_chain_id.to_string(),
            &to_chain_id.to_string(),
            &order.input_token,
            &order.output_token,
            from_address,
            to_address,
            &order.amount,
        )
        .await
        .map_err(SwapOrderError::LiFiError)?;

    match quote.transaction_request {
        Some(transaction_request) => {
            if transaction_request.is_solana() {
                Ok(SwapOrderTransaction::Solana(transaction_request.data))
            } else {
                Ok(SwapOrderTransaction::Evm(
                    transaction_request
                        .to_json_rpc()
                        .map_err(SwapOrderError::SerializeError)?,
                ))
            }
        }
        None => Err(SwapOrderError::NoTransactionRequest),
    }
}

// Helper function that actually performs the swap operation
async fn try_solana_swap_order_to_transaction(
    order: &SwapOrder,
    pubkey: &str,
) -> Result<SwapOrderTransaction, SwapOrderError> {
    let quote = Jupiter::fetch_quote(
        &order.input_token,
        &order.output_token,
        order
            .amount
            .parse::<u64>()
            .map_err(|e| SwapOrderError::InvalidAmount(anyhow::anyhow!(e)))?,
    )
    .await
    .map_err(SwapOrderError::JupiterError)?;

    let tx = Jupiter::swap(
        quote,
        &Pubkey::from_str(pubkey).map_err(|e| SwapOrderError::InvalidPubkey(anyhow::anyhow!(e)))?,
    )
    .await
    .map_err(SwapOrderError::JupiterError)?;

    Ok(SwapOrderTransaction::Solana(transaction_to_base64(&tx)?))
}

pub fn transaction_to_base64<T: Serialize>(transaction: &T) -> Result<String, SwapOrderError> {
    let serialized = bincode::serialize(transaction)
        .map_err(|e| SwapOrderError::SerializeError(anyhow::anyhow!(e)))?;
    Ok(base64encode(&serialized))
}

#[cfg(test)]
mod tests {
    use crate::engine::{
        constants::{TEST_ADDRESS_EVM, TEST_ADDRESS_SOL},
        execute::ensure_approvals,
    };

    use blockhash_cache::{inject_blockhash_into_encoded_tx, BLOCKHASH_CACHE};
    use lifi::quote::TransactionRequest;

    use super::*;

    async fn test_swap_generic(
        input_token: &str,
        output_token: &str,
        amount: &str,
        from_chain_caip2: &str,
        to_chain_caip2: &str,
    ) {
        tracing_subscriber::fmt::init();
        dotenv::dotenv().ok();

        let swap_order = SwapOrder {
            amount: amount.to_string(),
            input_token: input_token.to_string(),
            output_token: output_token.to_string(),
            from_chain_caip2: from_chain_caip2.to_string(),
            to_chain_caip2: to_chain_caip2.to_string(),
        };

        let lifi_api_key: Option<String> = match std::env::var("LIFI_API_KEY") {
            Ok(val) => Some(val),
            Err(_) => None,
        };

        let lifi = lifi::LiFi::new(lifi_api_key);
        let transaction = swap_order_to_transaction(
            &swap_order,
            &lifi,
            Some(TEST_ADDRESS_EVM.to_string()),
            Some(TEST_ADDRESS_SOL.to_string()),
        )
        .await
        .unwrap();

        let privy = std::sync::Arc::new(privy::Privy::new(
            privy::config::PrivyConfig::from_env().unwrap(),
        ));
        let privy_tx = match transaction {
            SwapOrderTransaction::Solana(encoded_tx) => {
                tracing::info!("Solana transaction: {:#?}", encoded_tx);
                let tx = if from_chain_caip2 == Caip2::SOLANA && to_chain_caip2 == Caip2::SOLANA {
                    // For Solana-to-Solana, inject blockhash
                    inject_blockhash_into_encoded_tx(
                        &encoded_tx,
                        &BLOCKHASH_CACHE.get_blockhash().await.unwrap().to_string(),
                    )
                    .unwrap()
                } else {
                    encoded_tx
                };

                privy::tx::PrivyTransaction {
                    user_id: "did:privy:cm6cxky3i00ondmuatkemmffm".to_string(),
                    address: TEST_ADDRESS_SOL.to_string(),
                    from_chain_caip2: swap_order.from_chain_caip2.clone(),
                    to_chain_caip2: swap_order.to_chain_caip2.clone(),
                    evm_transaction: None,
                    solana_transaction: Some(tx),
                }
            }
            SwapOrderTransaction::Evm(transaction) => {
                tracing::info!("EVM transaction: {:#?}", transaction);
                privy::tx::PrivyTransaction {
                    user_id: "did:privy:cm6cxky3i00ondmuatkemmffm".to_string(),
                    address: TEST_ADDRESS_EVM.to_string(),
                    from_chain_caip2: swap_order.from_chain_caip2.clone(),
                    to_chain_caip2: swap_order.to_chain_caip2.clone(),
                    evm_transaction: Some(transaction),
                    solana_transaction: None,
                }
            }
        };

        tracing::info!("Executing transaction: {:#?}", privy_tx);
        if is_evm(&swap_order.from_chain_caip2) {
            tracing::info!("Ensuring approvals");
            ensure_approvals(
                privy_tx.evm_transaction.as_ref().unwrap()["to"]
                    .as_str()
                    .unwrap(),
                &swap_order,
                &privy_tx,
                privy.clone(),
            )
            .await
            .unwrap();
        }
        let res = privy.execute_transaction(privy_tx).await.unwrap();
        tracing::info!("Transaction result: {:#?}", res);
    }

    // sol works!
    #[tokio::test]
    #[ignore]
    async fn test_sol_to_sol() {
        test_swap_generic(
            "So11111111111111111111111111111111111111112",  // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // SOL
            "10000",                                        // 0.001 SOL
            Caip2::SOLANA,
            Caip2::SOLANA,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_sol_to_bsc() {
        test_swap_generic(
            "11111111111111111111111111111111",           // SOL
            "0x0000000000000000000000000000000000000000", // BNB
            &(3. * 10e6).to_string(),                     // 0.03 SOL
            Caip2::SOLANA,
            Caip2::BSC,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_sol_to_base() {
        test_swap_generic(
            "11111111111111111111111111111111",           // SOL
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // USDC arbitrum
            &(1. * 10e6).to_string(),                     // 0.01 SOL
            Caip2::SOLANA,
            Caip2::ARBITRUM,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_sol_to_arbitrum() {
        test_swap_generic(
            "11111111111111111111111111111111",           // SOL
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // USDC arbitrum
            &(1. * 10e6).to_string(),                     // 0.01 SOL
            Caip2::SOLANA,
            Caip2::ARBITRUM,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_bsc_to_bsc() {
        test_swap_generic(
            "0x0000000000000000000000000000000000000000", // BNB
            "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", // CAKE
            "1000000000000000",                           // 0.001 BNB
            Caip2::BSC,
            Caip2::BSC,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_arb_to_sol() {
        test_swap_generic(
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // USDC arbitrum
            "11111111111111111111111111111111",           // SOL
            "1000000",                                    // 1 USDC
            Caip2::ARBITRUM,
            Caip2::SOLANA,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_arb_to_base() {
        test_swap_generic(
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // USDC arbitrum
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC base
            "1000000",                                    // 1 USDC
            Caip2::ARBITRUM,
            Caip2::BASE,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_base_to_base() {
        test_swap_generic(
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC base
            "0x0000000000000000000000000000000000000000", // ETH
            "1000000",                                    // 1 USDC
            Caip2::BASE,
            Caip2::BASE,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_base_to_bsc() {
        test_swap_generic(
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC base
            "0x0000000000000000000000000000000000000000", // ETH
            "2000000",                                    // 2 USDC
            Caip2::BASE,
            Caip2::BSC,
        )
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_bsc_to_bsc_simple() {
        // Load env vars and init tracing
        dotenv::dotenv().ok();
        tracing_subscriber::fmt::init();

        // let _lifi = lifi::LiFi::new(None);

        // // Get the transaction
        // let transaction =r
        //     swap_order_to_transaction(&swap_order, &lifi, TEST_ADDRESS_EVM, TEST_ADDRESS_SOL)
        //         .await
        //         .expect("Failed to get transaction");

        let privy = privy::Privy::new(privy::config::PrivyConfig::from_env().unwrap());

        let value = format!("0x{}", hex::encode((10e8 as u64).to_le_bytes().to_vec()));
        let gas_limit = format!("0x{}", hex::encode((1000000 as u64).to_le_bytes().to_vec()));
        let gas_price = format!(
            "0x{}",
            hex::encode((1000000000 as u64).to_le_bytes().to_vec())
        );
        println!("Value: {:#?}", value);

        // Execute the transaction
        let res = privy
            .execute_transaction(privy::tx::PrivyTransaction {
                user_id: "test-user".to_string(),
                address: TEST_ADDRESS_EVM.to_string(),
                from_chain_caip2: Caip2::BSC.to_string(),
                to_chain_caip2: Caip2::BSC.to_string(),
                evm_transaction: Some(
                    TransactionRequest {
                        data: "0x".to_string(),
                        chain_id: Some(serde_json::Number::from(56)),
                        from: Some(TEST_ADDRESS_EVM.to_string()),
                        gas_limit: Some(gas_limit),
                        gas_price: Some(gas_price),
                        to: Some(TEST_ADDRESS_EVM.to_string()),
                        value: Some(value), // 0.001 bnb
                    }
                    .to_json_rpc()
                    .unwrap(),
                ),
                solana_transaction: None,
            })
            .await;

        match res {
            Ok(result) => {
                tracing::info!("Transaction successful: {:#?}", result);
            }
            Err(e) => {
                tracing::error!("Transaction failed: {:#?}", e);
                panic!("Transaction execution failed: {}", e);
            }
        }
    }
}
