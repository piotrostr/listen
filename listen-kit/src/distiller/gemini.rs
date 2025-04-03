use std::sync::Arc;

use anyhow::Result;

use super::analyst::{
    AnalystAgent, AnalystError, AnalystType, ChartAnalystAgent,
    TwitterAnalystAgent, WebAnalystAgent,
};
use super::preambles;
use crate::agents::delegate::delegate_to_agent;
use crate::common::gemini_agent_builder;
use crate::reasoning_loop::Model;
use crate::signer::SignerContext;

pub struct GeminiAnalystAgent {
    pub agent: Model,
    pub locale: String,
    pub analyst_type: AnalystType,
}

#[async_trait::async_trait]
impl AnalystAgent for GeminiAnalystAgent {
    fn locale(&self) -> &str {
        &self.locale
    }

    fn agent_type(&self) -> AnalystType {
        self.analyst_type
    }
}

// Twitter analyst implementation for Gemini
#[async_trait::async_trait]
impl TwitterAnalystAgent for GeminiAnalystAgent {
    async fn analyze_twitter(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let prompt_text = if let Some(intent) = intent {
            format!(
                "query: {}\nresponse: {}\nintent: {}",
                query, response, intent
            )
        } else {
            format!("query: {}\nresponse: {}", query, response)
        };

        delegate_to_agent(
            prompt_text,
            self.agent.clone(),
            "twitter_analyst".to_string(),
            SignerContext::current().await,
            false, // with stdout
        )
        .await
        .map_err(|e| AnalystError::DelegateError(e.to_string()))
    }
}

// Chart analyst implementation for Gemini
#[async_trait::async_trait]
impl ChartAnalystAgent for GeminiAnalystAgent {
    async fn analyze_chart(
        &self,
        candlesticks: &[crate::data::listen_api_tools::Candlestick],
        interval: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let candlesticks_json = serde_json::to_string(candlesticks)
            .map_err(|_| AnalystError::SerializationError)?;

        let prompt_text = if self.locale == "zh" {
            let base = format!(
                "分析这些K线图数据，时间间隔为{}:\n{}",
                interval, candlesticks_json
            );
            if let Some(intent) = intent {
                format!("{}意图是{}", base, intent)
            } else {
                base
            }
        } else {
            let base = format!(
                "Analyze these candlesticks with interval {}:\n{}",
                interval, candlesticks_json
            );
            if let Some(intent) = intent {
                format!("{}Intent is: {}", base, intent)
            } else {
                base
            }
        };

        delegate_to_agent(
            prompt_text,
            self.agent.clone(),
            "chart_analyst".to_string(),
            SignerContext::current().await,
            false, // with stdout
        )
        .await
        .map_err(|e| AnalystError::DelegateError(e.to_string()))
    }
}
// Web analyst implementation for Gemini
#[async_trait::async_trait]
impl WebAnalystAgent for GeminiAnalystAgent {
    async fn analyze_web(
        &self,
        url: &str,
        content: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let prompt_text = if self.locale == "zh" {
            let base =
                format!("分析以下网页内容，URL为{}:\n{}", url, content);
            if let Some(intent) = intent {
                format!("{}意图是{}", base, intent)
            } else {
                base
            }
        } else {
            let base = format!(
                "Analyze this web content from URL {}:\n{}",
                url, content
            );
            if let Some(intent) = intent {
                format!("{}Intent is: {}", base, intent)
            } else {
                base
            }
        };

        delegate_to_agent(
            prompt_text,
            self.agent.clone(),
            "web_analyst".to_string(),
            SignerContext::current().await,
            false, // with stdout
        )
        .await
        .map_err(|e| AnalystError::DelegateError(e.to_string()))
    }
}

pub fn make_gemini_analyst(
    analyst_type: AnalystType,
    locale: &str,
    preamble: Option<String>,
) -> GeminiAnalystAgent {
    let default_preamble = match (analyst_type, locale) {
        (AnalystType::Twitter, "zh") => preambles::TWITTER_ZH,
        (AnalystType::Twitter, _) => preambles::TWITTER_EN,
        (AnalystType::Chart, "zh") => preambles::CHART_ZH,
        (AnalystType::Chart, _) => preambles::CHART_EN,
        (AnalystType::Web, "zh") => preambles::WEB_ZH,
        (AnalystType::Web, _) => preambles::WEB_EN,
    };

    let agent = gemini_agent_builder()
        .preamble(&preamble.unwrap_or(default_preamble.to_string()))
        .build();

    GeminiAnalystAgent {
        agent: Model::Gemini(Arc::new(agent)),
        locale: locale.to_string(),
        analyst_type,
    }
}
