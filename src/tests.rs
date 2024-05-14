#![cfg(test)]
use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::{
    constants, provider::Provider, raydium, tx_parser, util::must_get_env,
};

#[test]
#[ignore = "This test requires a live network connection"]
fn test_get_pricing() {
    let url = "https://api.mainnet-beta.solana.com";
    let provider = crate::provider::Provider::new(url.to_string());
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

#[test]
#[ignore = "This test requires a live network connection"]
fn test_parse_new_pool() {
    let new_pool_tx_signature: &str = "2nkbEdznrqqoXyxcrYML8evHtAKcNTurBBXGWACS6cxJDHYGosgVdy66gaqHzgtRWWH13bzMF4kovSEQUVYdDPku";
    let provider = Provider::new(must_get_env("RPC_URL"));
    let tx = provider.get_tx(new_pool_tx_signature).unwrap();
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
    let provider = Provider::new(must_get_env("RPC_URL"));
    assert!(provider.sanity_check(&mint).await.unwrap().0 == false);
    // michi
    let mint = Pubkey::from_str("5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp")
        .unwrap();
    assert!(provider.sanity_check(&mint).await.unwrap().0 == true);
}

#[test]
fn test_get_burn_pct() {
    let lp_mint =
        Pubkey::from_str("CcX9jxEAxeBvMjtLCYeMaecr7XGmoB9aiY3wePeXmEu5")
            .unwrap();
    let amm_pool =
        Pubkey::from_str("G5ts2NDTcAhzowLqWTrVN6NcxKoAwrXHg3uPTyskfksd")
            .unwrap();
    let rpc_client =
        RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let result = raydium::get_calc_result(&rpc_client, &amm_pool).unwrap();
    let burn_pct =
        raydium::get_burn_pct(&rpc_client, &lp_mint, result).unwrap();

    assert!(burn_pct == 100.);
}
