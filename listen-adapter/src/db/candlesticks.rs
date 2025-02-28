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

#[derive(PartialEq, Eq, Debug)]
pub enum CandlestickInterval {
    FifteenSeconds,
    ThirtySeconds,
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    FourHours,
    OneDay,
}

impl CandlestickInterval {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "15s" => Ok(CandlestickInterval::FifteenSeconds),
            "30s" => Ok(CandlestickInterval::ThirtySeconds),
            "1m" => Ok(CandlestickInterval::OneMinute),
            "5m" => Ok(CandlestickInterval::FiveMinutes),
            "15m" => Ok(CandlestickInterval::FifteenMinutes),
            "30m" => Ok(CandlestickInterval::ThirtyMinutes),
            "1h" => Ok(CandlestickInterval::OneHour),
            "4h" => Ok(CandlestickInterval::FourHours),
            "1d" => Ok(CandlestickInterval::OneDay),
            _ => Err(anyhow::anyhow!("Invalid interval: {}", s)),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            CandlestickInterval::FifteenSeconds => "15 SECOND".to_string(),
            CandlestickInterval::ThirtySeconds => "30 SECOND".to_string(),
            CandlestickInterval::OneMinute => "1 MINUTE".to_string(),
            CandlestickInterval::FiveMinutes => "5 MINUTE".to_string(),
            CandlestickInterval::FifteenMinutes => "15 MINUTE".to_string(),
            CandlestickInterval::ThirtyMinutes => "30 MINUTE".to_string(),
            CandlestickInterval::OneHour => "1 HOUR".to_string(),
            CandlestickInterval::FourHours => "4 HOUR".to_string(),
            CandlestickInterval::OneDay => "1 DAY".to_string(),
        }
    }
}

impl serde::Serialize for CandlestickInterval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for CandlestickInterval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        CandlestickInterval::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl ClickhouseDb {
    pub async fn get_candlesticks(
        &self,
        mint: &str,
        interval: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Candlestick>> {
        let interval_seconds = match interval {
            "15 SECOND" => 15,
            "30 SECOND" => 30,
            "1 MINUTE" => 60,
            "5 MINUTE" => 300,
            "15 MINUTE" => 900,
            "30 MINUTE" => 1800,
            "1 HOUR" => 3600,
            "4 HOUR" => 14400,
            "1 DAY" => 86400,
            _ => return Err(anyhow::anyhow!("Invalid interval")),
        };

        let query = format!(
            r#"
            SELECT
                intDiv(timestamp, {interval_seconds}) * {interval_seconds} as interval_timestamp,
                argMin(price, timestamp) as open,
                max(price) as high,
                min(price) as low,
                argMax(price, timestamp) as close,
                sum(swap_amount) as volume
            FROM price_updates
            WHERE pubkey = '{mint}'
            GROUP BY interval_timestamp
            ORDER BY interval_timestamp DESC
            LIMIT {limit}
            "#,
            limit = limit.unwrap_or(200)
        );

        let result = self
            .client
            .query(&query)
            .fetch_all::<(u64, f64, f64, f64, f64, f64)>()
            .await?;

        let mut candlesticks = result
            .into_iter()
            .map(|(timestamp, open, high, low, close, volume)| Candlestick {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            })
            .collect::<Vec<_>>();

        // Reverse to maintain chronological order (oldest first)
        candlesticks.reverse();

        Ok(candlesticks)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::make_db;

    use super::CandlestickInterval;
    use crate::routes::CandlestickParams;

    #[test]
    fn test_candlestick_interval() {
        let interval = CandlestickInterval::OneMinute;
        assert_eq!(interval.to_string(), "1 MINUTE");
        let interval = CandlestickInterval::FiveMinutes;
        assert_eq!(interval.to_string(), "5 MINUTE");
        let interval = CandlestickInterval::FifteenMinutes;
        assert_eq!(interval.to_string(), "15 MINUTE");
        let interval = CandlestickInterval::ThirtyMinutes;
        assert_eq!(interval.to_string(), "30 MINUTE");
    }

    #[test]
    fn test_deserialize_candlestick_params() {
        let payload = r#"{"mint": "not-important", "interval": "1m"}"#;
        let params: CandlestickParams = serde_json::from_str(payload).unwrap();
        assert_eq!(params.mint, "not-important");
        assert_eq!(params.interval, CandlestickInterval::OneMinute);
    }

    #[tokio::test]
    async fn test_get_candlesticks() {
        let db = make_db().unwrap();
        let candlesticks = db
            .get_candlesticks(
                "GJAFwWjJ3vnTsrQVabjBVK2TYB1YtRCQXRDfDgUnpump",
                &CandlestickInterval::OneMinute.to_string(),
                None,
            )
            .await
            .unwrap();
        println!("{:#?}", candlesticks);
        println!(
            "{:#?}",
            chrono::DateTime::from_timestamp(candlesticks.first().unwrap().timestamp as i64, 0)
        );
        println!(
            "{:#?}",
            chrono::DateTime::from_timestamp(candlesticks.last().unwrap().timestamp as i64, 0)
        );
        println!(
            "now: {:#?}",
            chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp(), 0)
        );
    }
}
