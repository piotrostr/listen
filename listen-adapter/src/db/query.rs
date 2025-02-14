use crate::db::{ClickhouseDb, PriceUpdate};
use anyhow::Result;

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
}
