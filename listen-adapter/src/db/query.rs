use crate::db::{ClickhouseDb, PriceUpdate};
use anyhow::Result;
use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Row, Serialize)]
pub struct OpenPrice {
    pub price: f64,
    pub timestamp: u64,
}

impl ClickhouseDb {
    pub async fn get_by_mint(&self, mint: &str) -> Result<Vec<PriceUpdate>> {
        let query = format!(
            r#"
            SELECT * FROM price_updates
            WHERE pubkey = '{mint}'
            ORDER BY timestamp DESC
            LIMIT 50
            "#
        );

        let result = self.client.query(&query).fetch_all::<PriceUpdate>().await?;

        Ok(result)
    }

    pub async fn generic_query(&self, sql: &str) -> Result<Vec<PriceUpdate>> {
        let result = self.client.query(sql).fetch_all::<PriceUpdate>().await?;

        Ok(result)
    }

    pub async fn get_24h_open_price(&self, mint: &str) -> Result<Option<OpenPrice>> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let start_time = current_time - 24 * 3600; // 24h ago

        let query = format!(
            r#"
            SELECT 
                price,
                timestamp
            FROM price_updates
            WHERE pubkey = '{mint}'
            AND timestamp >= {start_time}
            ORDER BY timestamp ASC
            LIMIT 1
            "#
        );

        let result = self
            .client
            .query(&query)
            .fetch_optional::<OpenPrice>()
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::make_db;

    #[tokio::test]
    async fn test_get_by_mint() {
        let db = make_db().unwrap();
        let result = db
            .get_by_mint("9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump")
            .await
            .unwrap();
        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_generic_query() {
        let db = make_db().unwrap();
        let result = db.generic_query("SELECT 1").await.unwrap();
        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_get_24h_open_price() {
        let db = make_db().unwrap();

        // Test existing token
        let result = db
            .get_24h_open_price("9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump")
            .await
            .unwrap();
        println!("24h open price for existing token: {:#?}", result);

        // Test non-existent token
        let result = db
            .get_24h_open_price("nonexistenttoken123123123")
            .await
            .unwrap();
        assert!(result.is_none());
        println!("24h open price for non-existent token: {:#?}", result);
    }
}
