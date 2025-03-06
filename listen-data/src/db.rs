use std::{sync::Arc, time::Duration};

use crate::price::PriceUpdate;
use anyhow::{Context, Result};
use clickhouse::inserter::Inserter;
use clickhouse::Client;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[async_trait::async_trait]
pub trait Database {
    fn new(
        database_url: &str,
        password: &str,
        user: &str,
        database: &str,
    ) -> Self
    where
        Self: Sized;
    async fn initialize(&mut self) -> Result<()>;

    async fn health_check(&self) -> Result<()>;

    async fn insert_price(&self, price: &PriceUpdate) -> Result<()>;
}

pub struct ClickhouseDb {
    client: Client,
    inserter: Option<Arc<RwLock<Inserter<PriceUpdate>>>>,
    is_initialized: bool,
    max_rows: u64,
}

impl ClickhouseDb {
    fn create_inserter(&self) -> Result<Inserter<PriceUpdate>> {
        Ok(self
            .client
            .inserter::<PriceUpdate>("price_updates")
            .context("failed to prepare price insert statement")?
            .with_timeouts(
                Some(Duration::from_secs(5)),
                Some(Duration::from_secs(20)),
            )
            .with_max_rows(self.max_rows)
            .with_max_bytes(1_000_000) // price update is roughly ~200 bytes
            .with_period(Some(Duration::from_secs(15))))
    }
}

#[async_trait::async_trait]
impl Database for ClickhouseDb {
    fn new(
        database_url: &str,
        password: &str,
        user: &str,
        database: &str,
    ) -> Self {
        let client = Client::default()
            .with_url(database_url)
            .with_password(password)
            .with_user(user)
            .with_database(database);

        info!("Connecting to ClickHouse at {}", database_url);
        Self {
            client,
            inserter: None,
            is_initialized: false,
            max_rows: 1000,
        }
    }

    async fn health_check(&self) -> Result<()> {
        debug!("clickhouse healthz");
        self.client
            .query("SELECT 1")
            .execute()
            .await
            .context("Failed to execute health check query")?;
        Ok(())
    }

    async fn initialize(&mut self) -> Result<()> {
        debug!("initializing clickhouse");
        self.client
            .query(
                r#"
                CREATE TABLE IF NOT EXISTS price_updates (
                    name String,
                    pubkey String,
                    price Float64,
                    market_cap Float64,
                    timestamp UInt64,
                    slot UInt64,
                    swap_amount Float64,
                    owner String,
                    signature String,
                    multi_hop Bool,
                    is_buy Bool,
                    is_pump Bool,
                    INDEX idx_mints (name, pubkey) TYPE minmax GRANULARITY 1
                ) 
                ENGINE = MergeTree()
                ORDER BY (name, pubkey, timestamp)
                "#,
            )
            .execute()
            .await
            .context("Failed to create price_updates table")?;

        self.inserter = Some(Arc::new(RwLock::new(self.create_inserter()?)));
        self.is_initialized = true;

        Ok(())
    }

    /// insert_price uses a batched writer to avoid spamming writes
    /// it is configurable at the initializer
    async fn insert_price(&self, price: &PriceUpdate) -> Result<()> {
        debug!("inserting price: {}", price.signature);

        let mut inserter = self
            .inserter
            .as_ref()
            .expect("inserter not initialized")
            .write()
            .await;

        inserter
            .write(price)
            .context("Failed to write price to insert buffer")?;

        let pending = inserter.pending();
        debug!("Pending: {} rows ({} bytes)", pending.rows, pending.bytes);

        if pending.rows >= self.max_rows {
            let stats = inserter.commit().await?;
            info!("Committed {} rows ({} bytes)", stats.rows, stats.bytes);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::util::make_db;

    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let db = make_db().await.unwrap();
        db.health_check().await.unwrap();
    }
}
