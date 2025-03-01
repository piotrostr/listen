pub mod chains;
pub mod client;
pub mod connections;
pub mod quote;
pub mod tokens;
pub mod tools;

use anyhow::Result;

use chains::ChainsResponse;
use client::LiFiClient;
use connections::ConnectionsResponse;
use tokens::{Token, TokensResponse};
use tools::ToolsResponse;

use crate::client::LiFiClientError;

use self::quote::{Order, QuoteResponse};

pub struct LiFi {
    client: LiFiClient,
}

#[derive(Debug, thiserror::Error)]
pub enum LiFiError {
    #[error("LiFi client error: {0}")]
    ClientError(LiFiClientError),
}

impl LiFi {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: LiFiClient::new(api_key),
        }
    }

    pub async fn get_chains(&self) -> Result<ChainsResponse, LiFiError> {
        self.client
            .get("/chains", &[])
            .await
            .map_err(LiFiError::ClientError)
    }

    pub async fn get_tools(
        &self,
        chains: &[String], // TODO possibly an enum here
    ) -> Result<ToolsResponse, LiFiError> {
        self.client
            .get("/tools", &[("chains", &chains.join(","))])
            .await
            .map_err(LiFiError::ClientError)
    }

    pub async fn get_tokens(
        &self,
        chains: &str,
        chain_types: Option<&str>,
        min_price_usd: Option<f64>,
    ) -> Result<TokensResponse, LiFiError> {
        let mut params = vec![("chains", chains)];
        if let Some(chain_types) = chain_types {
            params.push(("chainTypes", chain_types));
        }
        let price_string;
        if let Some(price) = min_price_usd {
            price_string = price.to_string();
            params.push(("minPriceUSD", &price_string));
        }
        self.client
            .get("/tokens", &params)
            .await
            .map_err(LiFiError::ClientError)
    }

    pub async fn get_token(&self, chain: &str, token: &str) -> Result<Token, LiFiError> {
        self.client
            .get("/token", &[("chain", chain), ("token", token)])
            .await
            .map_err(LiFiError::ClientError)
    }

    // TODO some params were ommited for brevity
    pub async fn get_connections(
        &self,
        from_chain: Option<&str>,
        to_chain: Option<&str>,
        from_token: Option<&str>,
        to_token: Option<&str>,
    ) -> Result<ConnectionsResponse, LiFiError> {
        let mut params = vec![];
        if let Some(from_chain) = from_chain {
            params.push(("fromChain", from_chain));
        }
        if let Some(to_chain) = to_chain {
            params.push(("toChain", to_chain));
        }
        if let Some(from_token) = from_token {
            params.push(("fromToken", from_token));
        }
        if let Some(to_token) = to_token {
            params.push(("toToken", to_token));
        }
        self.client
            .get("/connections", &params)
            .await
            .map_err(LiFiError::ClientError)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_quote(
        &self,
        from_chain: &str,
        to_chain: &str,
        from_token: &str,
        to_token: &str,
        from_address: &str,
        to_address: &str,
        from_amount_with_decimals: &str,
    ) -> Result<QuoteResponse, LiFiError> {
        let order = Order::Fastest.to_string();
        let params = vec![
            ("fromChain", from_chain),
            ("toChain", to_chain),
            ("fromToken", from_token),
            ("toToken", to_token),
            ("fromAddress", from_address),
            ("toAddress", to_address),
            ("fromAmount", from_amount_with_decimals),
            ("order", &order),
        ];

        self.client
            .get("/quote", &params)
            .await
            .map_err(LiFiError::ClientError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_chains() {
        let lifi = LiFi::new(None);
        let chains = lifi.get_chains().await;
        assert!(chains.is_ok(), "{:?}", chains);
        println!(
            "{:#?}",
            chains
                .unwrap()
                .chains
                .iter()
                .map(|c| serde_json::json!({"id":c.id, "name":c.name}))
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_get_tools() {
        let lifi = LiFi::new(None);
        let tools = lifi.get_tools(&["sol".to_string()]).await;
        assert!(tools.is_ok(), "{:?}", tools);
    }

    #[tokio::test]
    async fn test_get_tokens() {
        let lifi = LiFi::new(None);
        let tokens = lifi.get_tokens("sol", None, Some(0.1)).await;
        assert!(tokens.is_ok(), "{:?}", tokens);
    }

    #[tokio::test]
    async fn test_get_token() {
        let lifi = LiFi::new(None);
        let token = lifi.get_token("sol", "listen").await;
        assert!(token.is_ok(), "{:?}", token);
    }

    #[tokio::test]
    async fn test_get_connections() {
        let lifi = LiFi::new(None);
        let connections = lifi
            .get_connections(Some("sol"), Some("eth"), Some("USDC"), Some("ETH"))
            .await;
        assert!(connections.is_ok(), "{:?}", connections);
    }

    #[tokio::test]
    async fn test_get_quote_base() {
        let lifi = LiFi::new(None);
        let quote = lifi
            .get_quote(
                "arb",
                "8453",
                "USDC",
                "0x532f27101965dd16442E59d40670FaF5eBB142E4",
                "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770",
                "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770",
                "1000000000",
            )
            .await;
        assert!(quote.is_ok(), "{:?}", quote);
        println!("{:#?}", quote.unwrap());
    }

    #[tokio::test]
    async fn test_get_quote_sol() {
        let lifi = LiFi::new(None);
        let quote = lifi
            .get_quote(
                "1151111081099710", // sol
                "42161",            // arb
                "So11111111111111111111111111111111111111112",
                "ETH",
                "aiamaErRMjbeNmf2b8BMZWFR3ofxrnZEf2mLKp935fM",
                "0x2fAA30d5EdDF1e4fa126aEdA79159878D58A2438",
                "1000000000",
            )
            .await;
        assert!(quote.is_ok(), "{:?}", quote);
    }

    #[tokio::test]
    async fn test_get_quote_evm() {
        let lifi = LiFi::new(None);
        let quote = lifi
            .get_quote(
                "arb",
                "sol",
                "USDC",
                "USDC",
                "0x2fAA30d5EdDF1e4fa126aEdA79159878D58A2438",
                "aiamaErRMjbeNmf2b8BMZWFR3ofxrnZEf2mLKp935fM",
                "1000000000",
            )
            .await;
        assert!(quote.is_ok(), "{:?}", quote);
        println!("{:#?}", quote.unwrap());
    }

    #[tokio::test]
    async fn test_arb_to_base() {
        tracing_subscriber::fmt::init();
        let lifi = LiFi::new(None);
        let quote = lifi
            .get_quote(
                "42161",                                      // arb
                "8453",                                       // base
                "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // usdc
                "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // usdc
                "0x2fAA30d5EdDF1e4fa126aEdA79159878D58A2438",
                "0x2fAA30d5EdDF1e4fa126aEdA79159878D58A2438",
                "1000000000",
            )
            .await;

        assert!(quote.is_ok(), "{:?}", quote);
    }

    #[tokio::test]
    async fn test_display_quote() {
        let lifi = LiFi::new(None);
        let quote = lifi
            .get_quote(
                "arb",
                "sol",
                "USDC",
                "USDC",
                "0x2fAA30d5EdDF1e4fa126aEdA79159878D58A2438",
                "aiamaErRMjbeNmf2b8BMZWFR3ofxrnZEf2mLKp935fM",
                "1000000000",
            )
            .await;
        assert!(quote.is_ok(), "{:?}", quote);
        let quote = quote.unwrap();
        let summary = quote.summary();
        println!("{:#?}", summary);
        println!("{:#?}", quote);
    }
}
