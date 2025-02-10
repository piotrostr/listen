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

    async fn commit_price_updates(&self) -> Result<()>;
}

pub struct ClickhouseDb {
    client: Client,
    inserter: Option<Arc<RwLock<Inserter<PriceUpdate>>>>,
    transaction_count: Arc<RwLock<u32>>,
    is_initialized: bool,
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
            transaction_count: Arc::new(RwLock::new(0)),
            is_initialized: false,
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
            .await
            .context("Failed to create price_updates table")?;

        self.inserter = Some(Arc::new(RwLock::new(
            self.client
                .inserter::<PriceUpdate>("price_updates")
                .context("Failed to prepare price insert statement")?
                .with_timeouts(
                    Some(Duration::from_secs(5)),
                    Some(Duration::from_secs(20)),
                )
                .with_max_bytes(50_000_000)
                .with_max_rows(750_000)
                .with_period(Some(Duration::from_secs(15))),
        )));

        self.is_initialized = true;

        Ok(())
    }

    async fn insert_price(&self, price: &PriceUpdate) -> Result<()> {
        debug!("inserting price: {}", price.signature);
        self.inserter
            .as_ref()
            .expect("inserter not initialized")
            .write()
            .await
            .write(price)
            .context("Failed to write price to insert buffer")?;

        let mut count = self.transaction_count.write().await;
        *count += 1;

        if *count >= 500 {
            info!("Transaction count reached {}, triggering commit", *count);
            *count = 0;
            drop(count);

            self.commit_price_updates().await?;
        }

        Ok(())
    }

    async fn commit_price_updates(&self) -> Result<()> {
        debug!("committing price updates");
        let stats = self
            .inserter
            .as_ref()
            .expect("inserter not initialized")
            .write()
            .await
            .commit()
            .await
            .context("Failed to commit price updates")?;

        info!("Committed {} rows ({} bytes)", stats.rows, stats.bytes);

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
