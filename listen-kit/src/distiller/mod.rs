use anyhow::Result;

pub mod analyst;
pub mod deepseek;
pub mod gemini;
pub mod preambles;

#[cfg(test)]
mod tests;

use analyst::{Analyst, AnalystError};

impl Analyst {
    // Methods for each analyst type
    pub async fn analyze_twitter(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        if let Some(agent) = &self.twitter_agent {
            agent.analyze_twitter(query, response, intent).await
        } else {
            Err(AnalystError::UnsupportedOperation)
        }
    }

    pub async fn analyze_chart(
        &self,
        candlesticks: &[crate::data::Candlestick],
        interval: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        if let Some(agent) = &self.chart_agent {
            agent.analyze_chart(candlesticks, interval, intent).await
        } else {
            Err(AnalystError::UnsupportedOperation)
        }
    }

    pub async fn analyze_web(
        &self,
        url: &str,
        content: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        if let Some(agent) = &self.web_agent {
            agent.analyze_web(url, content, intent).await
        } else {
            Err(AnalystError::UnsupportedOperation)
        }
    }
}
