use crate::{price::PriceUpdate, util::must_get_env};
use anyhow::Result;
use clickhouse::Client;
use tracing::{debug, info};

#[async_trait::async_trait]
pub trait Database {
    fn new() -> Self
    where
        Self: Sized;
    async fn initialize(&self) -> Result<()>;

    async fn health_check(&self) -> Result<()>;

    async fn insert_price(&self, price: &PriceUpdate) -> Result<()>;
}

pub struct ClickhouseDb {
    client: Client,
}

#[async_trait::async_trait]
impl Database for ClickhouseDb {
    fn new() -> Self {
        let database_url = must_get_env("CLICKHOUSE_URL");
        let password = must_get_env("CLICKHOUSE_PASSWORD");
        let user = must_get_env("CLICKHOUSE_USER");
        let database = must_get_env("CLICKHOUSE_DATABASE");

        let client = Client::default()
            .with_url(database_url.as_str())
            .with_password(password)
            .with_user(user)
            .with_database(database);

        info!("Connecting to ClickHouse at {}", database_url);
        Self { client }
    }

    async fn health_check(&self) -> Result<()> {
        debug!("clickhouse healthz");
        self.client.query("SELECT 1").execute().await?;
        Ok(())
    }

    async fn initialize(&self) -> Result<()> {
        debug!("initializing clickhouse");
        self.client
            .query(
                r#"
                CREATE TABLE IF NOT EXISTS price_updates (
                    name String,
                    pubkey String,
                    price Float64,
                    market_cap Float64,
                    timestamp DateTime64(0),
                    slot UInt64,
                    swap_amount Float64,
                    owner String,
                    signature String,
                    base_in Bool,
                    INDEX idx_mints (name, pubkey) TYPE minmax GRANULARITY 1
                ) 
                ENGINE = MergeTree()
                ORDER BY (name, pubkey, timestamp)
                "#,
            )
            .execute()
            .await?;

        Ok(())
    }

    async fn insert_price(&self, price: &PriceUpdate) -> Result<()> {
        debug!("inserting price: {}", price.signature);
        let mut insert = self.client.insert::<PriceUpdate>("price_updates")?;
        insert.write(price).await?;
        insert.end().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let db = ClickhouseDb::new();
        db.health_check().await.unwrap();
    }
}
