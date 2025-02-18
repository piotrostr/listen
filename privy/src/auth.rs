use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::{
    types::{LinkedAccount, PrivyClaims, User, WalletAccount},
    Privy,
};

#[derive(Clone)]
pub struct UserSession {
    pub user_id: String,
    pub session_id: String,
    pub wallet_address: String,
    pub pubkey: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PrivyAuthError {
    #[error("[Privy] Failed to validate access token")]
    ValidateAccessTokenError(#[from] jsonwebtoken::errors::Error),
    #[error("[Privy] Failed to get user by id")]
    GetUserByIdRequestError(#[from] reqwest::Error),
    #[error("[Privy] Failed to get user by id")]
    GetUserByIdFailed(anyhow::Error),
    #[error("[Privy] Failed to parse user data")]
    ParseUserError(#[from] serde_json::Error),
    #[error("[Privy] Failed to find wallet")]
    FindWalletError(anyhow::Error),
}

impl Privy {
    pub async fn authenticate_user(
        &self,
        access_token: &str,
    ) -> Result<UserSession, PrivyAuthError> {
        let claims = self.validate_access_token(access_token)?;
        let user = self.get_user_by_id(&claims.user_id).await?;

        let mut session = UserSession {
            user_id: user.id,
            session_id: claims.session_id,
            wallet_address: String::new(),
            pubkey: String::new(),
        };

        let solana_wallet = find_wallet(&user.linked_accounts, "solana", "privy")
            .map_err(PrivyAuthError::FindWalletError)?;
        session.pubkey = solana_wallet.address.clone();

        let evm_wallet = find_wallet(&user.linked_accounts, "ethereum", "privy")
            .map_err(PrivyAuthError::FindWalletError)?;

        session.wallet_address = evm_wallet.address.clone();

        Ok(session)
    }

    pub fn validate_access_token(&self, access_token: &str) -> Result<PrivyClaims, PrivyAuthError> {
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&["privy.io"]);
        validation.set_audience(&[self.config.app_id.clone()]);

        let key = DecodingKey::from_ec_pem(self.config.verification_key.as_bytes())
            .map_err(PrivyAuthError::ValidateAccessTokenError)?;

        let token_data = decode::<PrivyClaims>(access_token, &key, &validation)
            .map_err(PrivyAuthError::ValidateAccessTokenError)?;

        Ok(token_data.claims)
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User, PrivyAuthError> {
        let url = format!("https://auth.privy.io/api/v1/users/{}", user_id);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(PrivyAuthError::GetUserByIdRequestError)?;

        if !response.status().is_success() {
            return Err(PrivyAuthError::GetUserByIdFailed(anyhow!(
                "Failed to get user data: {}",
                response.status()
            )));
        }
        let text = response.text().await?;
        Ok(serde_json::from_str(&text)?)
    }
}

fn find_wallet<'a>(
    linked_accounts: &'a [LinkedAccount],
    chain_type: &str,
    wallet_client: &str,
) -> Result<&'a WalletAccount> {
    linked_accounts
        .iter()
        .find_map(|account| match account {
            LinkedAccount::Wallet(wallet) => {
                if wallet.delegated
                    && wallet.chain_type == chain_type
                    && wallet.wallet_client == wallet_client
                {
                    Some(wallet)
                } else {
                    None
                }
            }
            _ => None,
        })
        .ok_or_else(|| anyhow!("Could not find a delegated {} wallet", chain_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_access_token() {
        dotenv::dotenv().ok();
        let privy = Privy::new(crate::config::PrivyConfig::from_env().unwrap());
        let claims = privy.validate_access_token("eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IkNPbGxUWHB2R3Jua3hXUThpbDA4V0paVjhvU3Y5c3g1dG5jNHMxS3libW8ifQ.eyJzaWQiOiJjbTc5Ymg0MDkwMXN6MTNqMTdnamtsd254IiwiaXNzIjoicHJpdnkuaW8iLCJpYXQiOjE3Mzk4OTUzNTUsImF1ZCI6ImNtNmM3aWZxZDAwYXI1Mm0xcXhmZ2Jra24iLCJzdWIiOiJkaWQ6cHJpdnk6Y202Y3hreTNpMDBvbmRtdWF0a2VtbWZmbSIsImV4cCI6MTczOTg5ODk1NX0.6XEndM7e1ZBLrLm6mZxor2OJZVtqNYqVHwogYxN14Lv9hEpXcbGktmfBOby1VMa3NIbecFEsMbciW9uAHR384g");
        println!("claims: {:?}", claims);
        assert!(claims.is_ok());
    }
}
