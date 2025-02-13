use super::ClickhouseDb;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Candlestick {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl ClickhouseDb {
    pub async fn get_candlesticks(&self, mint: &str, interval: &str) -> Result<Vec<Candlestick>> {
        let query = format!(
            r#"
            SELECT
                toStartOfInterval(timestamp, INTERVAL {interval}) as interval_timestamp,
                argMin(price, timestamp) as open,
                max(price) as high,
                min(price) as low,
                argMax(price, timestamp) as close,
                sum(swap_amount) as volume
            FROM price_updates
            WHERE pubkey = '{mint}'
            GROUP BY interval_timestamp
            ORDER BY interval_timestamp ASC
            "#
        );

        let result = self
            .client
            .query(&query)
            .fetch_all::<(u64, f64, f64, f64, f64, f64)>()
            .await?;

        let candlesticks = result
            .into_iter()
            .map(|(timestamp, open, high, low, close, volume)| Candlestick {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            })
            .collect();

        Ok(candlesticks)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::make_db;

    #[tokio::test]
    async fn test_get_candlesticks() {
        let db = make_db().unwrap();
        let candlesticks = db
            .get_candlesticks("9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump", "1h")
            .await
            .unwrap();
        println!("{:?}", candlesticks);
    }
}
