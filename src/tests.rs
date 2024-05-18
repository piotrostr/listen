#![cfg(test)]
use std::str::FromStr;

use dotenv_codegen::dotenv;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;

use crate::{
    buyer::check_top_holders, constants, provider::Provider, tx_parser,
};

const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

#[test]
#[ignore = "This test requires a live network connection"]
fn test_get_pricing() {
    let provider = crate::provider::Provider::new(RPC_URL.to_string());
    let mint = "Fv17uvL3nsD4tBJaowdKz9SUsKFoxeZdcTuGTaKgyYQU";

    let pricing = tokio_test::block_on(provider.get_pricing(mint)).unwrap();
    assert!(pricing.data[mint].price > 0., "Price not found");
}

#[test]
#[ignore = "not used atm"]
fn test_parse_notional() {
    let tx =
        serde_json::from_reader(std::fs::File::open("mock/tx.json").unwrap())
            .unwrap();
    let sol_notional = crate::tx_parser::parse_notional(&tx).unwrap();
    assert!(1510000000 > sol_notional && sol_notional > 1500000000);
}

#[tokio::test]
#[ignore = "This test requires a live network connection"]
async fn test_parse_new_pool() {
    let new_pool_tx_signature: &str = "2nkbEdznrqqoXyxcrYML8evHtAKcNTurBBXGWACS6cxJDHYGosgVdy66gaqHzgtRWWH13bzMF4kovSEQUVYdDPku";
    let provider = Provider::new(RPC_URL.to_string());
    let tx = provider.get_tx(new_pool_tx_signature).await.unwrap();
    println!("{}", serde_json::to_string_pretty(&tx).unwrap());
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
#[ignore = "This test requires a live network connection"]
async fn test_sanity_check() {
    // non-renounced freeze authority
    let mint = Pubkey::from_str("3jGenV1FXBQWKtviJUWXUwXFiA8TNV4QGF2n499HnJmw")
        .unwrap();
    let provider = Provider::new(RPC_URL.to_string());
    assert!(provider.sanity_check(&mint).await.unwrap().0 == false);
    // michi
    let mint = Pubkey::from_str("5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp")
        .unwrap();
    assert!(provider.sanity_check(&mint).await.unwrap().0 == true);
}

#[test]
fn test_parse_mint_acc() {
    let data = "DK9N1P4LsskfLtyXTYoeDi44sjaGgT3n8akj2pFAiqsfFhJyaPYhhVqC17vKirYk9vmh2kBf7jQeTKybRETHCMRv9dKQSufNqo457fnX1dZCGCo";
    let _ = Mint::unpack(bs58::decode(data).into_vec().unwrap().as_slice())
        .expect("unpack mint data");
}

#[tokio::test]
#[ignore = "requires a live network connection"]
async fn test_gets_top_holders() {
    let mint = Pubkey::from_str("D2oKMNHb94DSgvibQxCweZPrbFEhayKBQ5eaPMC4Dvnv")
        .unwrap();
    let ok = check_top_holders(
        &mint,
        &Provider::new(dotenv!("RPC_URL").to_string()),
    )
    .await
    .unwrap();
    assert_eq!(false, true);
}
