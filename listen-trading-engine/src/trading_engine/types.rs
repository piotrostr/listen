use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SignAndSendEvmTransactionRequest {
    pub address: String,
    pub chain_type: String, // Always "ethereum"
    pub method: String,     // Always "eth_sendTransaction"
    pub caip2: String,      // Format: "eip155:{chain_id}"
    pub params: SignAndSendEvmTransactionParams,
}

#[derive(Serialize)]
pub struct SignAndSendEvmTransactionParams {
    pub transaction: serde_json::Value,
}

// Request types for signing transactions
#[derive(Serialize)]
pub struct SignAndSendTransactionRequest {
    pub address: String,
    pub chain_type: String,
    pub method: String,
    pub caip2: String,
    pub params: SignAndSendTransactionParams,
}

#[derive(Serialize)]
pub struct SignAndSendTransactionParams {
    pub transaction: String,
    pub encoding: String,
}

#[derive(Deserialize)]
pub struct SignAndSendTransactionResponse {
    pub method: String,
    pub data: SignAndSendTransactionData,
}

#[derive(Deserialize)]
pub struct SignAndSendTransactionData {
    pub hash: String,
    pub caip2: String,
}
