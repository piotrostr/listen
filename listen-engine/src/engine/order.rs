use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use privy::caip2::Caip2;

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
    wallet_address: &str, // evm output
    pubkey: &str,         // solana output
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

#[cfg(test)]
mod tests {
    use crate::engine::constants::{TEST_ADDRESS_EVM, TEST_ADDRESS_SOL};

    use blockhash_cache::{inject_blockhash_into_encoded_tx, BLOCKHASH_CACHE};

    use super::*;

    #[tokio::test]
    async fn test_swap_from_swap_order() {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt::init();
        let swap_order = SwapOrder {
            amount: "1000000".to_string(),
            input_token: "11111111111111111111111111111111".to_string(),
            output_token: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string(), // usdc arbitrum
            from_chain_caip2: Caip2::SOLANA.to_string(),
            to_chain_caip2: Caip2::ARBITRUM.to_string(),
        };

        let lifi = lifi::LiFi::new(None);
        let transaction =
            swap_order_to_transaction(&swap_order, &lifi, TEST_ADDRESS_EVM, TEST_ADDRESS_SOL)
                .await
                .unwrap();
        let encoded_tx = match transaction {
            SwapOrderTransaction::Solana(transaction) => {
                tracing::info!("transaction: {:#?}", transaction);
                transaction
            }
            _ => panic!("Invalid transaction type"),
        };

        let privy = privy::Privy::new(privy::config::PrivyConfig::from_env().unwrap());
        let res = privy
            .execute_transaction(privy::tx::PrivyTransaction {
                user_id: "did:privy:cm6cxky3i00ondmuatkemmffm".to_string(),
                address: TEST_ADDRESS_SOL.to_string(),
                from_chain_caip2: swap_order.from_chain_caip2,
                to_chain_caip2: swap_order.to_chain_caip2,
                evm_transaction: None,
                solana_transaction: Some(encoded_tx),
            })
            .await
            .unwrap();
        tracing::info!("res: {:#?}", res);
    }

    #[tokio::test]
    // TODO fix this too
    async fn test_swap_from_swap_order_evm() {
        let swap_order = SwapOrder {
            amount: "100000".to_string(),
            input_token: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string(), // usdc arbitrum
            output_token: "11111111111111111111111111111111".to_string(),          // usdc solana
            from_chain_caip2: Caip2::ARBITRUM.to_string(),
            to_chain_caip2: Caip2::SOLANA.to_string(),
        };

        let lifi = lifi::LiFi::new(None);
        let transaction =
            swap_order_to_transaction(&swap_order, &lifi, TEST_ADDRESS_EVM, TEST_ADDRESS_SOL)
                .await
                .unwrap();
        let serialized_tx = match transaction {
            SwapOrderTransaction::Evm(transaction) => {
                tracing::info!("transaction: {:#?}", transaction);
                transaction
            }
            _ => panic!("Invalid transaction type"),
        };

        dotenv::dotenv().ok();
        let privy = privy::Privy::new(privy::config::PrivyConfig::from_env().unwrap());

        let res = privy
            .execute_transaction(privy::tx::PrivyTransaction {
                user_id: "did:privy:cm6cxky3i00ondmuatkemmffm".to_string(),
                address: TEST_ADDRESS_EVM.to_string(),
                from_chain_caip2: swap_order.from_chain_caip2,
                to_chain_caip2: swap_order.to_chain_caip2,
                evm_transaction: Some(serialized_tx),
                solana_transaction: None,
            })
            .await
            .unwrap();
        tracing::info!("res: {:#?}", res);
    }

    #[tokio::test]
    async fn test_sol_to_sol() {
        tracing_subscriber::fmt::init();
        let swap_order = SwapOrder {
            amount: "10000".to_string(),
            input_token: "11111111111111111111111111111111".to_string(),
            output_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            from_chain_caip2: Caip2::SOLANA.to_string(),
            to_chain_caip2: Caip2::SOLANA.to_string(),
        };

        let address = TEST_ADDRESS_SOL;

        let lifi = lifi::LiFi::new(None);
        let transaction = swap_order_to_transaction(&swap_order, &lifi, TEST_ADDRESS_EVM, address)
            .await
            .unwrap();
        let encoded_tx = match transaction {
            SwapOrderTransaction::Solana(transaction) => {
                tracing::info!("transaction: {:#?}", transaction);
                transaction
            }
            _ => panic!("Invalid transaction type"),
        };

        dotenv::dotenv().ok();
        let privy = privy::Privy::new(privy::config::PrivyConfig::from_env().unwrap());
        let privy_tx = privy::tx::PrivyTransaction {
            user_id: "did:privy:cm6cxky3i00ondmuatkemmffm".to_string(),
            address: TEST_ADDRESS_SOL.to_string(),
            from_chain_caip2: swap_order.from_chain_caip2.clone(),
            to_chain_caip2: swap_order.to_chain_caip2.clone(),
            evm_transaction: None,
            solana_transaction: Some(
                inject_blockhash_into_encoded_tx(
                    &encoded_tx,
                    &BLOCKHASH_CACHE.get_blockhash().await.unwrap().to_string(),
                )
                .unwrap(),
            ),
        };
        tracing::info!("privy_tx: {:#?}", privy_tx);
        privy.execute_transaction(privy_tx).await.unwrap();
    }
}
