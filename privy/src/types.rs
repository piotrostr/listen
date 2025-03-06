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
    #[serde(rename = "phone")]
    Phone(serde_json::Value),
    #[serde(rename = "unknown")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>, // Can be either "eip155:1" or "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp" format
    pub chain_type: String, // Can be "ethereum" or "solana"
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_user_with_solana_wallets() {
        let json = r#"{
            "id": "did:privy:cm7fb598p00zh10hue8hoiciy",
            "created_at": 1740174800,
            "linked_accounts": [
                {
                    "type": "wallet",
                    "address": "aiamaErRMjbeNmf2b8BMZWFR3ofxrnZEf2mLKp935fM",
                    "chain_type": "solana",
                    "wallet_client": "unknown",
                    "wallet_client_type": "phantom",
                    "connector_type": "solana_adapter",
                    "verified_at": 1740174800,
                    "first_verified_at": 1740174800,
                    "latest_verified_at": 1740304046
                },
                {
                    "type": "wallet",
                    "wallet_index": 0,
                    "wallet_client": "privy",
                    "wallet_client_type": "privy",
                    "connector_type": "embedded",
                    "imported": false,
                    "recovery_method": "privy",
                    "verified_at": 1740304050,
                    "first_verified_at": 1740304050,
                    "latest_verified_at": 1740304050,
                    "address": "0xfe86bbcA0048262853432e66c33F33dCAC331428",
                    "chain_id": "eip155:1",
                    "chain_type": "ethereum",
                    "delegated": true,
                    "id": "h11wchcq9jvks7xc48utw3xu"
                },
                {
                    "type": "wallet",
                    "wallet_index": 0,
                    "wallet_client": "privy",
                    "wallet_client_type": "privy",
                    "connector_type": "embedded",
                    "imported": false,
                    "recovery_method": "privy",
                    "verified_at": 1740304050,
                    "first_verified_at": 1740304050,
                    "latest_verified_at": 1740304050,
                    "address": "9zKvecYDKAW7G1mVLjY4ACjGLYpXAGGndUiGtPzodEoT",
                    "public_key": "9zKvecYDKAW7G1mVLjY4ACjGLYpXAGGndUiGtPzodEoT",
                    "chain_type": "solana",
                    "chain_id": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
                    "delegated": true,
                    "id": "atyh5yflm0h2pzqzv7islxun"
                }
            ],
            "mfa_methods": [],
            "has_accepted_terms": false,
            "is_guest": false
        }"#;

        let user: User = serde_json::from_str(json).expect("Should deserialize user data");

        assert_eq!(user.id, "did:privy:cm7fb598p00zh10hue8hoiciy");
        assert_eq!(user.linked_accounts.len(), 3);

        // Test first wallet (Phantom Solana wallet)
        if let LinkedAccount::Wallet(wallet) = &user.linked_accounts[0] {
            assert_eq!(
                wallet.address,
                "aiamaErRMjbeNmf2b8BMZWFR3ofxrnZEf2mLKp935fM"
            );
            assert_eq!(wallet.chain_type, "solana");
            assert_eq!(wallet.wallet_client_type, "phantom");
            assert!(wallet.chain_id.is_none()); // First wallet has no chain_id
        } else {
            panic!("First account should be a wallet");
        }

        // Test second wallet (Ethereum wallet)
        if let LinkedAccount::Wallet(wallet) = &user.linked_accounts[1] {
            assert_eq!(wallet.address, "0xfe86bbcA0048262853432e66c33F33dCAC331428");
            assert_eq!(wallet.chain_type, "ethereum");
            assert_eq!(wallet.chain_id.as_ref().unwrap(), "eip155:1");
        } else {
            panic!("Second account should be a wallet");
        }

        // Test third wallet (Privy Solana wallet)
        if let LinkedAccount::Wallet(wallet) = &user.linked_accounts[2] {
            assert_eq!(
                wallet.address,
                "9zKvecYDKAW7G1mVLjY4ACjGLYpXAGGndUiGtPzodEoT"
            );
            assert_eq!(wallet.chain_type, "solana");
            assert_eq!(
                wallet.chain_id.as_ref().unwrap(),
                "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
            );
            assert_eq!(
                wallet.public_key.as_ref().unwrap(),
                "9zKvecYDKAW7G1mVLjY4ACjGLYpXAGGndUiGtPzodEoT"
            );
        } else {
            panic!("Third account should be a wallet");
        }
    }
}
