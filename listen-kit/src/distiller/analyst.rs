use chrono::DateTime;

use crate::distiller::{
    deepseek::make_deepseek_analyst, gemini::make_gemini_analyst,
};

pub struct Analyst {
    pub twitter_agent: Option<Box<dyn TwitterAnalystAgent>>,
    pub chart_agent: Option<Box<dyn ChartAnalystAgent>>,
    pub web_agent: Option<Box<dyn WebAnalystAgent>>,
    pub locale: String,
}

// Create a general error type for analysts
#[derive(Debug, thiserror::Error)]
pub enum AnalystError {
    #[error("API key is not set")]
    ApiKeyNotSet,

    #[error("Model error")]
    PromptError(rig::completion::PromptError),

    #[error("Serialization error")]
    SerializationError,

    #[error("Unsupported operation for this analyst type")]
    UnsupportedOperation,

    #[error("Streaming error")]
    StreamingError(rig::completion::CompletionError),

    #[error("Delegate error")]
    DelegateError(String),

    #[error("Preprocess error: {0}")]
    PreprocessError(String),
}

// Common trait for all analyst types
#[async_trait::async_trait]
pub trait AnalystAgent: Send + Sync {
    fn locale(&self) -> &str;
    fn agent_type(&self) -> AnalystType;
}

// Enum to identify different analyst types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalystType {
    Twitter,
    Chart,
    Web,
}

// Twitter analyst trait
#[async_trait::async_trait]
pub trait TwitterAnalystAgent: AnalystAgent {
    async fn analyze_twitter(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

// Chart analyst trait
#[async_trait::async_trait]
pub trait ChartAnalystAgent: AnalystAgent {
    async fn analyze_chart(
        &self,
        candlesticks: &[crate::data::Candlestick],
        interval: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

// Web analyst trait
#[async_trait::async_trait]
pub trait WebAnalystAgent: AnalystAgent {
    async fn analyze_web(
        &self,
        url: &str,
        content: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

impl Analyst {
    pub fn new(locale: String) -> Self {
        Self {
            twitter_agent: None,
            chart_agent: None,
            web_agent: None,
            locale,
        }
    }

    pub fn with_twitter_analyst(
        mut self,
        agent: Box<dyn TwitterAnalystAgent>,
    ) -> Self {
        self.twitter_agent = Some(agent);
        self
    }

    pub fn with_chart_analyst(
        mut self,
        agent: Box<dyn ChartAnalystAgent>,
    ) -> Self {
        self.chart_agent = Some(agent);
        self
    }

    pub fn with_web_analyst(
        mut self,
        agent: Box<dyn WebAnalystAgent>,
    ) -> Self {
        self.web_agent = Some(agent);
        self
    }

    pub fn from_env_with_locale(
        locale: String,
    ) -> Result<Self, AnalystError> {
        let mut analyst = Self::new(locale.clone());

        let use_deepseek = false; // locale == "zh";

        if use_deepseek {
            let twitter_agent =
                make_deepseek_analyst(AnalystType::Twitter, &locale, None);
            let chart_agent =
                make_deepseek_analyst(AnalystType::Chart, &locale, None);
            let web_agent =
                make_deepseek_analyst(AnalystType::Web, &locale, None);

            analyst = analyst
                .with_twitter_analyst(Box::new(twitter_agent))
                .with_chart_analyst(Box::new(chart_agent))
                .with_web_analyst(Box::new(web_agent));
        } else {
            let twitter_agent =
                make_gemini_analyst(AnalystType::Twitter, &locale, None);
            let chart_agent =
                make_gemini_analyst(AnalystType::Chart, &locale, None);
            let web_agent =
                make_gemini_analyst(AnalystType::Web, &locale, None);

            analyst = analyst
                .with_twitter_analyst(Box::new(twitter_agent))
                .with_chart_analyst(Box::new(chart_agent))
                .with_web_analyst(Box::new(web_agent));
        }

        Ok(analyst)
    }
}

pub fn preprocess_candlesticks(
    candlesticks: &[crate::data::Candlestick],
) -> Result<Vec<serde_json::Value>, AnalystError> {
    let mut humanized_candlesticks = vec![];
    for candlestick in candlesticks {
        humanized_candlesticks.push(serde_json::json!({
            "timestamp": humanize_timestamp(candlestick.timestamp)?,
            "open": candlestick.open,
            "high": candlestick.high,
            "low": candlestick.low,
            "close": candlestick.close,
            "volume": candlestick.volume,
        }));
    }

    Ok(humanized_candlesticks)
}

pub fn humanize_timestamp(timestamp: u64) -> Result<String, AnalystError> {
    let datetime = if let Some(datetime) =
        DateTime::from_timestamp(timestamp as i64, 0)
    {
        datetime
    } else {
        return Err(AnalystError::PreprocessError(
            "Failed to convert timestamp to datetime".to_string(),
        ));
    };
    Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}
