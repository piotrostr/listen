use crate::{
    types::{Secp256k1SignRequest, Secp256k1SignResponse},
    Privy,
};
use anyhow::{anyhow, Result};

#[derive(Debug, thiserror::Error)]
pub enum PrivySignError {
    #[error("Failed to sign hash: {0}")]
    SignHashError(anyhow::Error),
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
}

impl Privy {
    pub async fn secp256k1_sign(
        &self,
        wallet_id: String,
        hash: String,
    ) -> Result<String, PrivySignError> {
        tracing::info!(?wallet_id, ?hash, "Signing hash with secp256k1");

        let request = Secp256k1SignRequest {
            chain_type: "ethereum".to_string(),
            method: "secp256k1_sign".to_string(),
            params: crate::types::Secp256k1SignParams { hash },
        };

        let response = self
            .client
            .post(&format!(
                "https://api.privy.io/v1/wallets/{}/rpc",
                wallet_id
            ))
            .json(&request)
            .send()
            .await
            .map_err(PrivySignError::RequestError)?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(PrivySignError::RequestError)?;
            return Err(PrivySignError::SignHashError(anyhow!(
                "Failed to sign hash: {}",
                error_text
            )));
        }

        let result: Secp256k1SignResponse = response
            .json()
            .await
            .map_err(|e| PrivySignError::SignHashError(e.into()))?;

        tracing::info!(
            ?result.method,
            ?result.data.signature,
            ?result.data.encoding,
            "Hash signed successfully",
        );
        Ok(result.data.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_WALLET_ID: &str = "k0pq0k5an1fvo35m5gm3wn8d";

    #[tokio::test]
    async fn test_secp256k1_sign() {
        dotenv::dotenv().ok();
        let privy = Privy::new(crate::config::PrivyConfig::from_env().unwrap());
        let result = privy
            .secp256k1_sign(
                TEST_WALLET_ID.to_string(),
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            )
            .await
            .unwrap();
        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 132); // 0x + 130 hex chars
    }
}
