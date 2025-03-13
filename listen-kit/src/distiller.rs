use anyhow::Result;

use rig::completion::Prompt;
use rig::providers::gemini::completion::CompletionModel as GeminiCompletionModel;

// Add import for DeepSeek
use rig::providers::deepseek::DeepSeekCompletionModel;

pub type GeminiAgent = rig::agent::Agent<GeminiCompletionModel>;
// Add DeepSeekAgent type
pub type DeepSeekAgent = rig::agent::Agent<DeepSeekCompletionModel>;

pub const DEFAULT_PREAMBLE: &str =
    "Your job is to extract the most relevant content from an
    Twitter API response and provide a summary. Be sure to take into account
    things like mindshare, the likes, retweets.
    1-500 likes - not a lot
    500-1k likes - some engagement
    1k-20k likes - decent engagement
    20k-100k likes - high engagement
    views:
    1-1000 views - not a lot
    1k-5k views - some engagement
    5k-20k views - decent engagement
    20k-100k views - high engagement
    If the profile has a blockchain address in the bio (solana public key,
    ethereum address), be sure to include it in the summary
    Good summary is to the point, enscapsulates the most important information and is not overly excessive
    Through providing tweet IDs and profile names format @username/tweet_id, it is possible to continue the analysis further
";

pub const DEFAULT_PREAMBLE_ZH: &str = "你的任务是从一个推特API响应中提取最相关的内容
，并提供一个总结。确保考虑到以下因素：
- 关注度
- 点赞数
- 转发数
- 评论数
- 用户互动
请用中文回答我接下来的所有问题。

1-500 likes - 没有太多关注
500-1k likes - 一些互动
1k-20k likes - 中等关注
20k-100k likes - 高关注

1-1000 views - 没有太多关注
1k-5k views - 一些互动
5k-20k views - 中等关注
20k-100k views - 高关注

如果用户在个人简介中包含区块链地址（solana 公钥，以太坊地址），请务必在总结中包含它。
通过提供推特ID和用户名，可以继续分析。
总结要简洁，抓住最重要的信息，不要过于冗长。
";
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
