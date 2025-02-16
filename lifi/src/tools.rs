use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Serialize, Deserialize, Debug)]
pub struct ToolsResponse {
    pub exchanges: Vec<Exchange>,
    pub bridges: Vec<Bridge>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub key: String,
    pub name: String,
    pub logo_uri: Option<String>,
    pub supported_chains: Vec<Number>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bridge {
    pub key: String,
    pub name: String,
    pub logo_uri: Option<String>,
    pub supported_chains: Vec<ChainSupport>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChainSupport {
    pub from_chain_id: Number,
    pub to_chain_id: Number,
}
