use super::analyst::{
    AnalystAgent, AnalystError, AnalystType, ChartAnalystAgent,
    TwitterAnalystAgent, WebAnalystAgent,
};
use super::preambles;
use anyhow::Result;
use rig::completion::Prompt;
use rig::providers::deepseek::DeepSeekCompletionModel;
pub type DeepSeekAgent = rig::agent::Agent<DeepSeekCompletionModel>;

pub struct DeepSeekAnalystAgent {
    agent: DeepSeekAgent,
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
        let prompt_text = if let Some(intent) = intent {
            format!(
                "query: {}\nresponse: {}\nintent: {}",
                query, response, intent
            )
        } else {
            format!("query: {}\nresponse: {}", query, response)
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
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

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
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

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
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

    let agent = rig::providers::deepseek::Client::from_env()
        .agent(rig::providers::deepseek::DEEPSEEK_CHAT)
        .preamble(&preamble.unwrap_or(default_preamble.to_string()))
        .build();

    DeepSeekAnalystAgent {
        agent,
        locale: locale.to_string(),
        analyst_type,
    }
}
