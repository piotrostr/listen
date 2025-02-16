use super::tokens::Token;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionsResponse {
    pub connections: Vec<Connection>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub from_chain_id: i64,
    pub to_chain_id: i64,
    pub from_tokens: Vec<Token>,
    pub to_tokens: Vec<Token>,
}
