use super::ClickhouseDb;
use anyhow::Result;

impl ClickhouseDb {
    pub async fn get_candlesticks(&self, token: &str, interval: &str) -> Result<()> {
        // TODO: Implement candlestick query
        Ok(())
    }
}
