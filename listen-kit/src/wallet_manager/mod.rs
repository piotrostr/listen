pub mod config;
pub mod kv_store;
pub mod types;
pub mod util;

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use solana_sdk::transaction::Transaction;

use config::PrivyConfig;
use kv_store::{KVStore, RedisKVStore, Wallet};
use types::{
    CreateWalletRequest, CreateWalletResponse, PrivyClaims,
    SignAndSendTransactionParams, SignAndSendTransactionRequest,
    SignAndSendTransactionResponse,
};

use util::{create_http_client, transaction_to_base64};

// TODO add idempotency keys management and wallet persistence with remote and redundant KV store
pub struct WalletManager<S: KVStore> {
    privy_config: PrivyConfig,
    http_client: reqwest::Client,
    kv_store: S,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct UserSession {
    pub(crate) user_id: String,
    pub(crate) wallet_id: String,
    pub(crate) access_token: String,
    pub(crate) session_id: String,
    pub(crate) wallet_address: String,
}

/// WalletManager currently only supports Solana
impl WalletManager<RedisKVStore> {
    pub fn new(privy_config: PrivyConfig) -> Self {
        let http_client = create_http_client(&privy_config);
        Self {
            privy_config,
            http_client,
            kv_store: RedisKVStore::new(),
        }
    }

    pub async fn create_wallet(&self) -> Result<CreateWalletResponse> {
        let request = CreateWalletRequest {
            chain_type: "solana".to_string(),
        };

        let response = self
            .http_client
            .post("https://api.privy.io/v1/wallets")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to create wallet: {} - {}",
                response.status(),
                response.text().await?
            ));
        }

        Ok(response.json().await?)
    }

    pub async fn authenticate_user(
        &self,
        access_token: &str,
    ) -> Result<UserSession> {
        let claims = self.validate_access_token(access_token)?;
        if let Some(Wallet {
            wallet_id,
            wallet_address,
        }) = self.kv_store.get_wallet(&claims.user_id).await?
        {
            Ok(UserSession {
                user_id: claims.user_id,
                wallet_id,
                access_token: access_token.to_string(),
                session_id: claims.session_id,
                wallet_address,
            })
        } else {
            let wallet = self.create_wallet().await?;
            self.kv_store
                .set_wallet(
                    &claims.user_id,
                    Wallet {
                        wallet_id: wallet.id.clone(),
                        wallet_address: wallet.address.clone(),
                    },
                )
                .await?;
            Ok(UserSession {
                user_id: claims.user_id,
                wallet_id: wallet.id,
                access_token: access_token.to_string(),
                session_id: claims.session_id,
                wallet_address: wallet.address,
            })
        }

        // ideally wanna get that from privy
        // if let Some(wallet_id) = self.get_wallet_from_claims(&claims).await? {
        //     Ok(UserSession {
        //         user_id: claims.user_id,
        //         wallet_id,
        //         access_token: access_token.to_string(),
        //         session_id: claims.session_id,
        //         wallet_address: "".to_string(), // TODO get wallet address
        //     })
        // } else {
        //     let wallet = self.create_wallet().await?;
        //     println!("{}", serde_json::to_string(&wallet)?);
        //     Ok(UserSession {
        //         user_id: claims.user_id,
        //         wallet_id: wallet.id,
        //         access_token: access_token.to_string(),
        //         session_id: claims.session_id,
        //         wallet_address: wallet.address,
        //     })
        // }
    }

    pub async fn sign_and_send_transaction(
        &self,
        session: &UserSession,
        transaction: Transaction,
    ) -> Result<String> {
        let request = SignAndSendTransactionRequest {
            method: "signAndSendTransaction".to_string(),
            caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
            params: SignAndSendTransactionParams {
                transaction: transaction_to_base64(transaction)?,
                encoding: "base64".to_string(),
            },
        };

        let response = self
            .http_client
            .post(format!(
                "https://api.privy.io/v1/wallets/{}/rpc",
                session.wallet_id
            ))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to sign transaction: {}",
                response.text().await?
            ));
        }

        let result: SignAndSendTransactionResponse = response.json().await?;
        Ok(result.data.hash)
    }

    pub fn validate_access_token(
        &self,
        access_token: &str,
    ) -> Result<PrivyClaims> {
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_issuer(&["privy.io"]);
        validation.set_audience(&[self.privy_config.app_id.clone()]);

        let key = DecodingKey::from_ec_pem(
            self.privy_config.verification_key.as_bytes(),
        )?;

        let token_data =
            decode::<PrivyClaims>(access_token, &key, &validation)
                .map_err(|_| anyhow!("Failed to authenticate"))?;

        Ok(token_data.claims)
    }

    // this endpoint returns information about the user, if it was possible it'd be nice to get the
    // wallet ID from here, let's see what privy says
    //
    // pub async fn get_wallet_from_claims(
    //     &self,
    //     claims: &PrivyClaims,
    // ) -> Result<Option<String>> {
    //     // Use the verified user_id to make the API call
    //     let url =
    //         format!("https://auth.privy.io/api/v1/users/{}", claims.user_id);

    //     let response = self.http_client.get(url).send().await?;

    //     if !response.status().is_success() {
    //         return Err(anyhow!(
    //             "Failed to get user data: {}",
    //             response.status()
    //         ));
    //     }
    //     let payload: serde_json::Value = response.json().await?;
    //     println!("{:?}", payload);
    //     let user_data: UserResponse = serde_json::from_value(payload)?;

    //     // let user_data: UserResponse = response.json().await?;

    //     // Return the first available wallet address (either regular or smart wallet)
    //     let wallet_address = user_data
    //         .wallet
    //         .map(|w| w.address)
    //         .or(user_data.smart_wallet.map(|w| w.address));

    //     Ok(wallet_address)
    // }
}
