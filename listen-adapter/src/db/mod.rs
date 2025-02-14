use anyhow::Result;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

pub mod candlesticks;
pub mod query;
pub mod top_tokens;

#[derive(Debug, Deserialize, Row, Serialize)]
pub struct PriceUpdate {
    pub name: String,
    pub pubkey: String,
    pub price: f64,
    pub market_cap: f64,
    pub timestamp: u64,
    pub slot: u64,
    pub swap_amount: f64,
    pub owner: String,
    pub signature: String,
    pub multi_hop: bool,
    pub is_buy: bool,
    pub is_pump: bool,
}

pub struct ClickhouseDb {
    client: Client,
}

impl ClickhouseDb {
    pub fn new(database_url: &str, password: &str, user: &str, database: &str) -> Self {
        let client = Client::default()
            .with_url(database_url)
            .with_password(password)
            .with_user(user)
            .with_database(database);

        Self { client }
    }

    pub async fn ping(&self) -> Result<()> {
        debug!("clickhouse healthz");
        self.client.query("SELECT 1").execute().await?;
        Ok(())
    }
}

pub fn is_local() -> bool {
    std::env::var("LOCAL").is_ok()
}

pub fn must_get_env(key: &str) -> String {
    std::env::var(key).expect(&format!("{} must be set", key))
}

pub fn make_db() -> Result<Arc<ClickhouseDb>> {
    let db = match is_local() {
        true => ClickhouseDb::new("http://localhost:8123", "default", "default", "default"),
        false => ClickhouseDb::new(
            must_get_env("CLICKHOUSE_URL").as_str(),
            must_get_env("CLICKHOUSE_USER").as_str(),
            must_get_env("CLICKHOUSE_PASSWORD").as_str(),
            must_get_env("CLICKHOUSE_DATABASE").as_str(),
        ),
    };
    Ok(Arc::new(db))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() -> Result<()> {
        let db = make_db()?;
        db.ping().await
    }
}
