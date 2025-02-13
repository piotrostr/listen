use super::ClickhouseDb;
use anyhow::Result;

impl ClickhouseDb {
    pub async fn get_candlesticks(&self, _token: &str, _interval: &str) -> Result<()> {
        todo!()
    }
}
