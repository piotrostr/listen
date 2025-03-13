use anyhow::Result;

use rig::completion::Prompt;
use rig::providers::gemini::completion::CompletionModel as GeminiCompletionModel;

// Add import for DeepSeek
use rig::providers::deepseek::DeepSeekCompletionModel;

pub type GeminiAgent = rig::agent::Agent<GeminiCompletionModel>;
// Add DeepSeekAgent type
pub type DeepSeekAgent = rig::agent::Agent<DeepSeekCompletionModel>;

pub const DEFAULT_PREAMBLE: &str = "You are a social media analytics expert. Your task is to analyze Twitter API responses and provide concise, data-driven summaries.

ENGAGEMENT METRICS:
Likes:
• Low (1-500): Limited reach
• Moderate (500-1K): Growing engagement
• High (1K-20K): Significant reach
• Viral (20K+): Exceptional performance

Views:
• Low (1-1K): Limited visibility
• Moderate (1K-5K): Growing visibility
• High (5K-20K): Significant visibility
• Viral (20K+): Exceptional visibility

KEY FOCUS AREAS:
1. Core Message: Extract the main point or announcement
2. Engagement Analysis: Quantify using the above metrics
3. Contract Addresses: Note any ETH/SOL addresses from bio
4. Key References: Include relevant usernames and tweet IDs (format: @username/tweet_id) for grounding
5. Provide future analysis points if applicable

OUTPUT FORMAT:
• Start with the most important insight
• Include relevant metrics and their interpretation
• Keep it concise
• End with actionable references if applicable";

pub const DEFAULT_PREAMBLE_ZH: &str = "您是社交媒体分析专家。您的任务是分析Twitter API响应并提供简洁的、数据驱动的总结。

互动指标：
点赞：
• 低 (1-500)：有限影响
• 中等 (500-1K)：正在增长
• 高 (1K-20K)：显著影响
• 病毒级 (20K+)：卓越表现

浏览量：
• 低 (1-1K)：有限可见度
• 中等 (1K-5K)：正在提升
• 高 (5K-20K)：显著可见度
• 病毒级 (20K+)：卓越表现

重点关注：
1. 核心信息：提取主要观点或公告
2. 互动分析：根据上述指标量化
3. 区块链信息：注意个人简介中的ETH/SOL地址
4. 关键引用：包含相关用户名和推文ID（格式：@用户名/推文ID）

输出格式：
• 以最重要的见解开始
• 包含相关指标及其解释
• 保持简洁（最多2-3段）
• 如果适用，以可操作的引用结束";

pub fn make_gemini_distiller(
    preamble: Option<String>,
) -> Result<GeminiAgent> {
    Ok(rig::providers::gemini::Client::from_env()
        .agent(rig::providers::gemini::completion::GEMINI_2_0_FLASH)
        .preamble(&preamble.unwrap_or(DEFAULT_PREAMBLE.to_string()))
        .build())
}

pub fn make_deepseek_distiller(
    preamble: Option<String>,
) -> Result<DeepSeekAgent> {
    Ok(rig::providers::deepseek::Client::from_env()
        .agent(rig::providers::deepseek::DEEPSEEK_CHAT)
        .preamble(&preamble.unwrap_or(DEFAULT_PREAMBLE_ZH.to_string()))
        .build())
}

// Add Send + Sync bounds to make the trait object thread-safe
#[async_trait::async_trait]
pub trait DistillerAgent: Send + Sync {
    async fn distill(
        &self,
        response: &serde_json::Value,
    ) -> Result<String, DistillerError>;
}

// Add this function to choose between Gemini and DeepSeek
pub fn make_distiller_agent(
    preamble: Option<String>,
) -> Result<Box<dyn DistillerAgent>> {
    // Default to Gemini, but you could add logic here to choose based on language
    let gemini_agent = make_gemini_distiller(preamble)?;
    Ok(Box::new(gemini_agent))
}

// Add a function to create a distiller that automatically chooses based on language
pub fn make_language_aware_distiller(
    preamble: Option<String>,
    use_chinese: bool,
) -> Result<Box<dyn DistillerAgent>> {
    if use_chinese {
        let deepseek_agent = make_deepseek_distiller(preamble)?;
        Ok(Box::new(deepseek_agent))
    } else {
        let gemini_agent = make_gemini_distiller(preamble)?;
        Ok(Box::new(gemini_agent))
    }
}

// Implementation for GeminiAgent
#[async_trait::async_trait]
impl DistillerAgent for GeminiAgent {
    async fn distill(
        &self,
        response: &serde_json::Value,
    ) -> Result<String, DistillerError> {
        self.prompt(response.to_string())
            .await
            .map_err(DistillerError::PromptError)
    }
}

// Implementation for DeepSeekAgent
#[async_trait::async_trait]
impl DistillerAgent for DeepSeekAgent {
    async fn distill(
        &self,
        response: &serde_json::Value,
    ) -> Result<String, DistillerError> {
        self.prompt(response.to_string())
            .await
            .map_err(DistillerError::PromptError)
    }
}

/// Distiller is a wrapper around multimodal models that allows to bring
/// understanding of assets, pass it a link to an image, video or large block of
/// text and receive a summary of the content.
pub struct Distiller {
    // Change to use our Promptable trait
    pub agent: Box<dyn DistillerAgent>,
}

#[derive(Debug, thiserror::Error)]
pub enum DistillerError {
    #[error("GEMINI_API_KEY is not set")]
    GeminiApiKeyNotSet,

    #[error("Model error")]
    PromptError(rig::completion::PromptError),
}

// TODO make this generic and find a better model, gemini struggles with chinese, deepseek is too slow
// wish grok3 was out...
impl Distiller {
    pub fn from_env() -> Result<Self, DistillerError> {
        Self::from_env_with_preamble(None)
    }

    pub fn from_env_with_preamble(
        preamble: Option<String>,
    ) -> Result<Self, DistillerError> {
        let agent = make_distiller_agent(preamble)
            .map_err(|_| DistillerError::GeminiApiKeyNotSet)?;
        Ok(Self { agent })
    }

    // Add a method to create distiller with language awareness
    pub fn from_env_with_language(
        preamble: Option<String>,
        use_chinese: bool,
    ) -> Result<Self, DistillerError> {
        let agent = make_language_aware_distiller(preamble, use_chinese)
            .map_err(|_| DistillerError::GeminiApiKeyNotSet)?;
        Ok(Self { agent })
    }

    // Update the distill method to use our Promptable trait
    pub async fn distill(
        &self,
        response: &serde_json::Value,
    ) -> Result<String, DistillerError> {
        self.agent.distill(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distiller() {
        let distiller = Distiller::from_env().unwrap();
        let result = distiller
            .distill(
                &std::fs::read_to_string("./debug/tweets_by_ids.json")
                    .unwrap()
                    .parse::<serde_json::Value>()
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{:#?}", result);
    }
}
