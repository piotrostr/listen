#![cfg(test)]
use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

use crate::{
    constants, provider::Provider, raydium::Raydium, tx_parser,
    util::must_get_env,
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
#[ignore = "This takes a long time and is likely to be deprecated"]
fn test_get_amm_pool_id() {
    let amm_pool_id =
        Pubkey::from_str("81kGW2fHNV5bSw9ChQ4HZT8SSPKkevtUqWtHPa5Jy7EQ")
            .unwrap();
    let mint = Pubkey::from_str("9Q9U4T2qMcXjs4G57RYUBKN2YA3JhbLzPtzMZ5QAvgQ3")
        .unwrap();
    let raydium = Raydium::new();
    let wsol = Pubkey::from_str(constants::SOLANA_PROGRAM_ID).unwrap();
    let provider = Provider::new(must_get_env("RPC_URL"));
    let got = raydium.get_amm_pool_id(&provider, &mint, &wsol);
    assert_eq!(got, amm_pool_id);
}

#[test]
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
