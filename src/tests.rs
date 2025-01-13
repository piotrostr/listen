#![cfg(test)]
use std::str::FromStr;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;

use crate::{
    buyer::check_top_holders, constants, provider::Provider, tx_parser,
    util::env,
};

#[test]
fn test_get_pricing() {
    let mint = "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump";

    let pricing = tokio_test::block_on(Provider::get_pricing(mint)).unwrap();
    println!("{:?}", pricing);
    assert!(pricing.data[mint].price > 0., "Price not found");
}

#[test]
fn test_parse_notional() {
    let tx =
        serde_json::from_reader(std::fs::File::open("mock/tx.json").unwrap())
            .unwrap();
    let sol_notional = crate::tx_parser::parse_notional(&tx).unwrap();
    assert!(1510000000 > sol_notional && sol_notional > 1500000000);
}

#[tokio::test]
async fn test_parse_new_pool() {
    let new_pool_tx_signature: &str =
        "2nkbEdznrqqoXyxcrYML8evHtAKcNTurBBXGWACS6cxJDHYGosgVdy66gaqHzgtRWWH13bzMF4kovSEQUVYdDPku";
    let rpc_client = RpcClient::new(env("RPC_URL"));
    let tx = Provider::get_tx(&rpc_client, new_pool_tx_signature)
        .await
        .unwrap();
    let new_pool_info = tx_parser::parse_new_pool(&tx).unwrap();
    assert_eq!(
        new_pool_info.amm_pool_id.to_string(),
        "8X6JvHBDTjB3hPQfNcHs5qvtZGwFVdwzj54SArvnw4NT".to_string(),
    );
    assert_eq!(
        new_pool_info.input_mint.to_string(),
        constants::SOLANA_PROGRAM_ID.to_string()
    );
    assert_eq!(
        new_pool_info.output_mint.to_string(),
        "9TMuCmQqMBaW8JRPGJEAuetJt94JVruuKVY8r8HvtYKd".to_string()
    );
}

#[tokio::test]
async fn test_sanity_check() {
    let rpc_client = RpcClient::new(env("RPC_URL"));
    // non-renounced freeze authority
    let mint =
        Pubkey::from_str("3jGenV1FXBQWKtviJUWXUwXFiA8TNV4QGF2n499HnJmw")
            .unwrap();
    assert!(!Provider::sanity_check(&rpc_client, &mint).await.unwrap().0);
    // michi
    let mint =
        Pubkey::from_str("5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp")
            .unwrap();
    assert!(Provider::sanity_check(&rpc_client, &mint).await.unwrap().0);
}

#[test]
fn test_parse_mint_acc() {
    let data = "DK9N1P4LsskfLtyXTYoeDi44sjaGgT3n8akj2pFAiqsfFhJyaPYhhVqC17vKirYk9vmh2kBf7jQeTKybRETHCMRv9dKQSufNqo457fnX1dZCGCo";
    let _ = Mint::unpack(bs58::decode(data).into_vec().unwrap().as_slice())
        .expect("unpack mint data");
}

#[tokio::test]
async fn test_gets_top_holders() {
    let mint =
        Pubkey::from_str("D2oKMNHb94DSgvibQxCweZPrbFEhayKBQ5eaPMC4Dvnv")
            .unwrap();
    let (_, ok, _) =
        check_top_holders(&mint, &RpcClient::new(env("RPC_URL")), false)
            .await
            .unwrap();
    assert!(ok);
}
