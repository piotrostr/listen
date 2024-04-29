#[cfg(test)]
mod tests {
    use crate::provider::Provider;

    #[test]
    fn test_get_pricing() {
        let url = "https://api.mainnet-beta.solana.com";
        let provider = Provider::new(url.to_string());
        let mint = "Fv17uvL3nsD4tBJaowdKz9SUsKFoxeZdcTuGTaKgyYQU";

        let pricing = tokio_test::block_on(provider.get_pricing(mint)).unwrap();
        assert!(pricing.data[mint].price > 0., "Price not found");
    }
}
