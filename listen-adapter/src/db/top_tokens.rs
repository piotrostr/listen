use super::ClickhouseDb;
use anyhow::Result;
use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct TopToken {
    pub name: String,
    pub pubkey: String,
    pub price: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
}

impl ClickhouseDb {
    pub async fn get_top_tokens(
        &self,
        limit: usize,
        min_volume: Option<f64>,
        min_market_cap: Option<f64>,
        max_market_cap: Option<f64>,
        time_range: Option<u64>,
        only_pumpfun_tokens: bool,
    ) -> Result<Vec<TopToken>> {
        let time_range = time_range.unwrap_or(86400); // 24h in seconds
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let start_time = current_time - time_range;

        let mut query = format!(
            r#"
            WITH 
                latest_prices AS (
                    SELECT 
                        name,
                        pubkey,
                        price,
                        market_cap,
                        timestamp,
                        is_pump
                    FROM price_updates
                    WHERE timestamp >= {start_time}
                    ORDER BY timestamp DESC
                    LIMIT 1 BY name, pubkey
                ),
                volumes AS (
                    SELECT
                        name,
                        pubkey,
                        sum(swap_amount) as volume_24h
                    FROM price_updates
                    WHERE timestamp >= {start_time}
                    GROUP BY name, pubkey
                ),
                price_changes AS (
                    SELECT
                        name,
                        pubkey,
                        (last_value(price) - first_value(price)) / first_value(price) * 100 as price_change_24h
                    FROM price_updates
                    WHERE timestamp >= {start_time}
                    GROUP BY name, pubkey
                )
            SELECT
                lp.name,
                lp.pubkey,
                lp.price,
                lp.market_cap,
                v.volume_24h,
                pc.price_change_24h
            FROM latest_prices lp
            LEFT JOIN volumes v ON lp.name = v.name AND lp.pubkey = v.pubkey
            LEFT JOIN price_changes pc ON lp.name = pc.name AND lp.pubkey = pc.pubkey
            "#
        );

        let mut conditions = Vec::new();

        if let Some(min_volume) = min_volume {
            conditions.push(format!("v.volume_24h >= {min_volume}"));
        }

        if let Some(min_market_cap) = min_market_cap {
            conditions.push(format!("lp.market_cap >= {min_market_cap}"));
        }

        if let Some(max_market_cap) = max_market_cap {
            conditions.push(format!("lp.market_cap <= {max_market_cap}"));
        }

        if only_pumpfun_tokens {
            conditions.push("is_pump = true".to_string());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(&format!(" ORDER BY v.volume_24h DESC LIMIT {}", limit));

        let result = self.client.query(&query).fetch_all::<TopToken>().await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::make_db;

    #[tokio::test]
    async fn test_get_top_tokens() -> Result<()> {
        let db = make_db()?;

        let tokens = db
            .get_top_tokens(
                10,              // limit
                Some(1000.0),    // min volume
                Some(100_000.0), // min market cap
                None,            // max market cap
                Some(24 * 3600), // 24h timeframe
                false,           // only show pumps
            )
            .await?;

        println!("Top 10 pump tokens:");
        for token in tokens {
            println!(
                "{}: price=${:.2}, mcap=${:.2}, vol=${:.2}, change={:.2}%",
                token.name, token.price, token.market_cap, token.volume_24h, token.price_change_24h
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_top_tokens_with_min_volume() -> Result<()> {
        let db = make_db()?;
        let tokens = db
            .get_top_tokens(10, Some(1000.0), None, None, None, false)
            .await?;
        println!("Top 10 tokens with min volume:");
        for token in tokens {
            println!(
                "{}: price=${:.2}, mcap=${:.2}, vol=${:.2}, change={:.2}%",
                token.name, token.price, token.market_cap, token.volume_24h, token.price_change_24h
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_top_tokens_with_min_market_cap() -> Result<()> {
        let db = make_db()?;
        let tokens = db
            .get_top_tokens(10, None, Some(100_000.0), None, None, false)
            .await?;
        println!("Top 10 tokens with min market cap:");
        for token in tokens {
            println!(
                "{}: price=${:.2}, mcap=${:.2}, vol=${:.2}, change={:.2}%",
                token.name, token.price, token.market_cap, token.volume_24h, token.price_change_24h
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_top_tokens_with_max_market_cap() -> Result<()> {
        let db = make_db()?;
        let tokens = db
            .get_top_tokens(10, None, None, Some(100_000.0), None, false)
            .await?;
        println!("Top 10 tokens with max market cap:");
        for token in tokens {
            println!(
                "{}: price=${:.2}, mcap=${:.2}, vol=${:.2}, change={:.2}%",
                token.name, token.price, token.market_cap, token.volume_24h, token.price_change_24h
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_top_tokens_with_only_pumpfun_tokens() -> Result<()> {
        let db = make_db()?;
        let tokens = db.get_top_tokens(10, None, None, None, None, true).await?;
        println!("Top 10 pumpfun tokens:");
        for token in tokens {
            println!(
                "{}: price=${:.2}, mcap=${:.2}, vol=${:.2}, change={:.2}%",
                token.name, token.price, token.market_cap, token.volume_24h, token.price_change_24h
            );
        }

        Ok(())
    }
}
