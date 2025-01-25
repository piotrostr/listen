use serde::{Deserialize, Serialize};

// Request types for signing transactions
#[derive(Serialize)]
pub struct SignAndSendTransactionRequest {
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
pub struct UserResponse {
    pub(crate) wallet: Option<WalletInfo>,
    pub(crate) smart_wallet: Option<WalletInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct WalletInfo {
    pub(crate) address: String,
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
