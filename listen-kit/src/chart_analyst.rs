use anyhow::Result;
use rig::completion::Prompt;
use rig::providers::deepseek::DeepSeekCompletionModel;
use rig::providers::gemini::completion::CompletionModel as GeminiCompletionModel;

use crate::data::Candlestick;

pub type GeminiAgent = rig::agent::Agent<GeminiCompletionModel>;
pub type DeepSeekAgent = rig::agent::Agent<DeepSeekCompletionModel>;

pub const DEFAULT_PREAMBLE: &str = "
Your job is to analyze candlestick chart data and provide meaningful insights about price patterns and market trends.
Focus on identifying key patterns such as:

1. Trend direction (bullish, bearish, or sideways)
2. Support and resistance levels
3. Common candlestick patterns (doji, hammer, engulfing patterns, etc.)
4. Volume analysis in relation to price movements
5. Potential reversal or continuation signals
6. Volatility assessment

Provide a concise summary that highlights the most important patterns and what they might indicate about future price direction.
";

pub const DEFAULT_PREAMBLE_ZH: &str = "
你的任务是分析K线图数据并提供有关价格模式和市场趋势的有意义见解。
重点识别以下关键模式：

1. 趋势方向（看涨、看跌或横盘）
2. 支撑位和阻力位
3. 常见K线形态（十字星、锤子线、吞没形态等）
4. 成交量与价格变动的关系分析
5. 潜在的反转或延续信号
6. 波动性评估

提供简明扼要的总结，突出最重要的模式以及它们可能预示的未来价格方向。
";

#[async_trait::async_trait]
pub trait ChartAnalystAgent: Send + Sync {
    fn locale(&self) -> &str;

    async fn analyze(
        &self,
        candlesticks: &[Candlestick],
        interval: &str,
    ) -> Result<String, ChartAnalystError>;
}

pub fn make_gemini_analyst(preamble: Option<String>) -> Result<GeminiAgent> {
    Ok(rig::providers::gemini::Client::from_env()
        .agent(rig::providers::gemini::completion::GEMINI_2_0_FLASH)
        .preamble(&preamble.unwrap_or(DEFAULT_PREAMBLE.to_string()))
        .build())
}

pub fn make_deepseek_analyst(
    preamble: Option<String>,
) -> Result<DeepSeekAgent> {
    Ok(rig::providers::deepseek::Client::from_env()
        .agent(rig::providers::deepseek::DEEPSEEK_CHAT)
        .preamble(&preamble.unwrap_or(DEFAULT_PREAMBLE_ZH.to_string()))
        .build())
}

pub fn make_language_aware_analyst(
    locale: String,
) -> Result<Box<dyn ChartAnalystAgent>> {
    if locale == "zh" {
        let deepseek_agent =
            make_deepseek_analyst(Some(DEFAULT_PREAMBLE_ZH.to_string()))?;
        Ok(Box::new(DeepSeekAgentWrapper {
            agent: deepseek_agent,
            locale: "zh".to_string(),
        }))
    } else {
        let gemini_agent =
            make_gemini_analyst(Some(DEFAULT_PREAMBLE.to_string()))?;
        Ok(Box::new(GeminiAgentWrapper {
            agent: gemini_agent,
            locale: "en".to_string(),
        }))
    }
}

// Wrapper structs to implement the trait
pub struct GeminiAgentWrapper {
    agent: GeminiAgent,
    locale: String,
}

pub struct DeepSeekAgentWrapper {
    agent: DeepSeekAgent,
    locale: String,
}

#[async_trait::async_trait]
impl ChartAnalystAgent for GeminiAgentWrapper {
    fn locale(&self) -> &str {
        &self.locale
    }

    async fn analyze(
        &self,
        candlesticks: &[Candlestick],
        interval: &str,
    ) -> Result<String, ChartAnalystError> {
        let candlesticks_json = serde_json::to_string(candlesticks)
            .map_err(|_| ChartAnalystError::SerializationError)?;

        let prompt_text = if self.locale == "zh" {
            format!(
                "分析这些K线图数据，时间间隔为{}:\n{}",
                interval, candlesticks_json
            )
        } else {
            format!(
                "Analyze these candlesticks with interval {}:\n{}",
                interval, candlesticks_json
            )
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(ChartAnalystError::PromptError)
    }
}

#[async_trait::async_trait]
impl ChartAnalystAgent for DeepSeekAgentWrapper {
    fn locale(&self) -> &str {
        &self.locale
    }

    async fn analyze(
        &self,
        candlesticks: &[Candlestick],
        interval: &str,
    ) -> Result<String, ChartAnalystError> {
        let candlesticks_json = serde_json::to_string(candlesticks)
            .map_err(|_| ChartAnalystError::SerializationError)?;

        let prompt_text = if self.locale == "zh" {
            format!(
                "分析这些K线图数据，时间间隔为{}:\n{}",
                interval, candlesticks_json
            )
        } else {
            format!(
                "Analyze these candlesticks with interval {}:\n{}",
                interval, candlesticks_json
            )
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(ChartAnalystError::PromptError)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChartAnalystError {
    #[error("API key is not set")]
    ApiKeyNotSet,

    #[error("Model error")]
    PromptError(rig::completion::PromptError),

    #[error("Serialization error")]
    SerializationError,
}

pub struct ChartAnalyst {
    pub agent: Box<dyn ChartAnalystAgent>,
}

impl ChartAnalyst {
    pub fn from_env() -> Result<Self, ChartAnalystError> {
        Self::from_env_with_locale("en".to_string())
    }

    pub fn from_env_with_locale(
        locale: String,
    ) -> Result<Self, ChartAnalystError> {
        let agent = make_language_aware_analyst(locale.clone())
            .map_err(|_| ChartAnalystError::ApiKeyNotSet)?;
        Ok(Self { agent })
    }

    pub async fn analyze(
        &self,
        candlesticks: &[Candlestick],
        interval: &str,
    ) -> Result<String, ChartAnalystError> {
        self.agent.analyze(candlesticks, interval).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Candlestick;

    #[tokio::test]
    async fn test_chart_analyst() {
        // Create some sample candlestick data
        let candlesticks = vec![
            Candlestick {
                timestamp: 1625097600,
                open: 35000.0,
                high: 35500.0,
                low: 34800.0,
                close: 35200.0,
                volume: 1000.0,
            },
            Candlestick {
                timestamp: 1625184000,
                open: 35200.0,
                high: 36000.0,
                low: 35100.0,
                close: 35800.0,
                volume: 1200.0,
            },
            // Add more sample data as needed
        ];

        let analyst = ChartAnalyst::from_env().unwrap();
        let result = analyst.analyze(&candlesticks, "1d").await.unwrap();
        println!("{}", result);
    }
}
