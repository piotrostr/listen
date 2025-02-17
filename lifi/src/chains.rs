use super::tokens::Token;
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainsResponse {
    pub chains: Vec<Chain>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    pub key: String,
    pub chain_type: Option<String>,
    pub name: String,
    pub coin: String,
    pub id: i64,
    pub mainnet: bool,
    pub logo_uri: Option<String>,
    pub tokenlist_url: Option<String>,
    pub faucet_urls: Option<Vec<String>>,
    pub multicall_address: Option<String>,
    pub metamask: Option<Metamask>,
    pub native_token: Option<Token>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Metamask {
    pub chain_id: String,
    pub block_explorer_urls: Vec<String>,
    pub chain_name: String,
    pub native_currency: NativeCurrency,
    pub rpc_urls: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NativeCurrency {
    pub name: String,
    pub symbol: String,
    pub decimals: Number,
}
