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

#[derive(Deserialize)]
pub struct SignAndSendEvmTransactionResponse {
    pub method: String,
    pub data: SignAndSendTransactionData,
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

// Response types for signed transactions
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivyClaims {
    #[serde(rename = "aud")]
    pub(crate) app_id: String,
    #[serde(rename = "exp")]
    pub(crate) expiration: i64,
    #[serde(rename = "iss")]
    pub(crate) issuer: String,
    #[serde(rename = "sub")]
    pub(crate) user_id: String,
    #[serde(rename = "iat")]
    pub(crate) issued_at: i64,
    #[serde(rename = "sid")]
    pub(crate) session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub created_at: i64,
    pub has_accepted_terms: bool,
    pub id: String,
    pub is_guest: bool,
    pub linked_accounts: Vec<LinkedAccount>,
    pub mfa_methods: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LinkedAccount {
    #[serde(rename = "email")]
    Email(EmailAccount),
    #[serde(rename = "wallet")]
    Wallet(WalletAccount),
    Unknown(serde_json::Map<String, serde_json::Value>),
}

#[derive(Serialize, Deserialize)]
pub struct EmailAccount {
    pub address: String,
    pub first_verified_at: u64,
    pub latest_verified_at: u64,
    pub verified_at: u64,
}

#[derive(Serialize, Deserialize)]
pub struct WalletAccount {
    pub address: String,
    pub chain_id: String,
    pub chain_type: String,
    pub connector_type: String,
    pub first_verified_at: u64,
    pub latest_verified_at: u64,
    pub verified_at: u64,
    pub wallet_client: String,
    pub wallet_client_type: String,
    // Optional fields
    #[serde(default)]
    pub delegated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_index: Option<u64>,
}

#[derive(Serialize)]
pub struct CreateWalletRequest {
    pub chain_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateWalletResponse {
    pub id: String,
    pub address: String,
    pub chain_type: String,
}
