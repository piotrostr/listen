use std::sync::Arc;

use crate::agents::delegate::delegate_to_agent;
use crate::common::deepseek_agent_builder;
use crate::reasoning_loop::Model;
use crate::signer::SignerContext;

use super::analyst::{
    AnalystAgent, AnalystError, AnalystType, ChartAnalystAgent,
    TwitterAnalystAgent, WebAnalystAgent,
};
use super::preambles;
use anyhow::Result;

pub struct DeepSeekAnalystAgent {
    agent: Model,
    locale: String,
    analyst_type: AnalystType,
}

#[async_trait::async_trait]
impl AnalystAgent for DeepSeekAnalystAgent {
    fn locale(&self) -> &str {
        &self.locale
    }

    fn agent_type(&self) -> AnalystType {
        self.analyst_type
    }
}

#[async_trait::async_trait]
impl TwitterAnalystAgent for DeepSeekAnalystAgent {
    async fn analyze_twitter(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let ctx = SignerContext::current().await;
        let user_id = ctx.user_id().unwrap_or_default();
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
            ctx,
            false, // with stdout
            user_id,
        )
        .await
        .map_err(|e| AnalystError::DelegateError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl ChartAnalystAgent for DeepSeekAnalystAgent {
    async fn analyze_chart(
        &self,
        candlesticks: &[crate::data::Candlestick],
        interval: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let ctx = SignerContext::current().await;
        let user_id = ctx.user_id().unwrap_or_default();
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
            ctx,
            false, // with stdout
            user_id,
        )
        .await
        .map_err(|e| AnalystError::DelegateError(e.to_string()))
    }
}

// Web analyst implementation for DeepSeek
#[async_trait::async_trait]
impl WebAnalystAgent for DeepSeekAnalystAgent {
    async fn analyze_web(
        &self,
        query: &str,
        content: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let ctx = SignerContext::current().await;
        let user_id = ctx.user_id().unwrap_or_default();
        let prompt_text = if self.locale == "zh" {
            let base = format!("分析以下网页内容，为{}:\n{}", query, content);
            if let Some(intent) = intent {
                format!("{}意图是{}", base, intent)
            } else {
                base
            }
        } else {
            let base = format!(
                "Analyze this web content from {}:\n{}",
                query, content
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
            ctx,
            false, // with stdout
            user_id,
        )
        .await
        .map_err(|e| AnalystError::DelegateError(e.to_string()))
    }
}

pub fn make_deepseek_analyst(
    analyst_type: AnalystType,
    locale: &str,
    preamble: Option<String>,
) -> DeepSeekAnalystAgent {
    let default_preamble = match (analyst_type, locale) {
        (AnalystType::Twitter, "zh") => preambles::TWITTER_ZH,
        (AnalystType::Twitter, _) => preambles::TWITTER_EN,
        (AnalystType::Chart, "zh") => preambles::CHART_ZH,
        (AnalystType::Chart, _) => preambles::CHART_EN,
        (AnalystType::Web, "zh") => preambles::WEB_ZH,
        (AnalystType::Web, _) => preambles::WEB_EN,
    };

    let agent = deepseek_agent_builder()
        .preamble(&preamble.unwrap_or(default_preamble.to_string()))
        .build();

    DeepSeekAnalystAgent {
        agent: Model::DeepSeek(Arc::new(agent)),
        locale: locale.to_string(),
        analyst_type,
    }
}
