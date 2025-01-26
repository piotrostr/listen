use anyhow::Result;
use redis::AsyncCommands;

#[async_trait::async_trait]
pub trait KVStore {
    fn new() -> Self
    where
        Self: Sized;
    async fn get_wallet(&self, user_id: &str) -> Result<Option<Wallet>>;
    async fn set_wallet(&self, user_id: &str, wallet: Wallet) -> Result<()>;
}

pub struct Wallet {
    pub(crate) wallet_address: String,
    pub(crate) wallet_id: String,
}

pub struct RedisKVStore {
    client: redis::Client,
}

#[async_trait::async_trait]
impl KVStore for RedisKVStore {
    fn new() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/")
            .expect("Failed to connect to Redis");
        Self { client }
    }

    async fn get_wallet(&self, user_id: &str) -> Result<Option<Wallet>> {
        let key = Self::make_wallet_key(user_id);
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let value: Option<String> = conn.get(&key).await?;

        match value {
            Some(json_str) => {
                let wallet: serde_json::Value =
                    serde_json::from_str(&json_str)?;
                Ok(Some(Wallet {
                    wallet_address: wallet["wallet_address"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    wallet_id: wallet["wallet_id"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                }))
            }
            None => Ok(None),
        }
    }

    async fn set_wallet(&self, user_id: &str, wallet: Wallet) -> Result<()> {
        let key = Self::make_wallet_key(user_id);
        let wallet_json = serde_json::to_string(&serde_json::json!({
            "wallet_address": wallet.wallet_address,
            "wallet_id": wallet.wallet_id,
        }))?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set(&key, wallet_json).await?;
        Ok(())
    }
}

impl RedisKVStore {
    fn make_wallet_key(user_id: &str) -> String {
        format!("wallet:solana:{}", user_id)
    }
}
