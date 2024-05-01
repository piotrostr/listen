#[test]
fn test_get_pricing() {
    let url = "https://api.mainnet-beta.solana.com";
    let provider = crate::provider::Provider::new(url.to_string());
    let mint = "Fv17uvL3nsD4tBJaowdKz9SUsKFoxeZdcTuGTaKgyYQU";

    let pricing = tokio_test::block_on(provider.get_pricing(mint)).unwrap();
    assert!(pricing.data[mint].price > 0., "Price not found");
}

#[test]
fn test_parse_notional() {
    let tx = serde_json::from_reader(std::fs::File::open("mock/tx.json").unwrap()).unwrap();
    let sol_notional = crate::tx_parser::parse_notional(&tx).unwrap();
    assert!(1510000000 > sol_notional && sol_notional > 1500000000);
}
