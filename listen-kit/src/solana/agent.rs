use super::tools::{
    DeployPumpFunToken, GetCurrentTime, GetQuote, GetSolBalance,
    GetSplTokenBalance, Swap,
};
use crate::agents::research::ViewImage;
use crate::common::{
    claude_agent_builder, deepseek_agent_builder, gemini_agent_builder,
    openai_agent_builder, ClaudeAgent, DeepSeekAgent, GeminiAgent,
    OpenAIAgent,
};
use crate::data::{
    AnalyzePageContent, FetchPriceActionAnalysis, FetchTokenMetadata,
    FetchTokenPrice, FetchTopTokens, FetchXPost, ResearchXProfile,
    SearchTweets, SearchWeb,
};
use crate::dexscreener::tools::SearchOnDexScreener;
use crate::faster100x::AnalyzeHolderDistribution;
use crate::lunarcrush::AnalyzeSentiment;
use crate::solana::advanced_orders::CreateAdvancedOrder;
use crate::solana::tools::AnalyzeRisk;
use crate::think::Think;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Features {
    pub autonomous: bool,
    pub deep_research: bool,
}

pub fn create_solana_agent(
    preamble: Option<String>,
    features: Features,
) -> ClaudeAgent {
    let preamble = preamble.unwrap_or(format!(
        "{}",
        "you are a solana trading agent that can also interact with pump.fun;"
    ));

    let mut agent = claude_agent_builder()
        .preamble(&preamble)
        .tool(GetQuote)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(FetchTopTokens)
        .tool(FetchTokenPrice)
        .tool(DeployPumpFunToken)
        .tool(FetchTokenMetadata)
        .tool(ResearchXProfile)
        .tool(ViewImage)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .tool(AnalyzeRisk)
        .tool(FetchPriceActionAnalysis)
        .tool(Think)
        .tool(GetCurrentTime)
        .tool(SearchWeb)
        .tool(AnalyzePageContent)
        .tool(AnalyzeSentiment);

    if features.autonomous {
        agent = agent.tool(Swap).tool(CreateAdvancedOrder);
    }

    agent.build()
}

pub fn create_solana_agent_gemini(
    preamble: Option<String>,
    features: Features,
) -> GeminiAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    let mut agent = gemini_agent_builder()
        .preamble(&preamble)
        .tool(GetQuote)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(FetchTopTokens)
        .tool(DeployPumpFunToken)
        .tool(FetchTokenMetadata)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .tool(AnalyzeRisk)
        .tool(FetchPriceActionAnalysis)
        .tool(Think)
        .tool(AnalyzeHolderDistribution)
        .tool(AnalyzeSentiment)
        .tool(GetCurrentTime)
        .tool(SearchWeb)
        .tool(ViewImage)
        .tool(AnalyzePageContent);

    if features.autonomous {
        agent = agent.tool(Swap).tool(CreateAdvancedOrder);
    }

    agent.build()
}

pub fn create_solana_agent_deepseek(
    preamble: Option<String>,
    features: Features,
) -> DeepSeekAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    let mut agent = deepseek_agent_builder()
        .preamble(&preamble)
        .tool(GetQuote)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(FetchTopTokens)
        .tool(DeployPumpFunToken)
        .tool(FetchTokenMetadata)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .tool(AnalyzeRisk)
        .tool(FetchPriceActionAnalysis)
        .tool(Think)
        .tool(AnalyzeHolderDistribution)
        .tool(AnalyzeSentiment)
        .tool(GetCurrentTime)
        .tool(SearchWeb)
        .tool(ViewImage)
        .tool(AnalyzePageContent);

    if features.autonomous {
        agent = agent.tool(Swap).tool(CreateAdvancedOrder);
    }

    agent.build()
}

pub fn create_solana_agent_openai(
    preamble: Option<String>,
    features: Features,
) -> OpenAIAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    let mut agent = openai_agent_builder()
        .preamble(&preamble)
        .tool(GetQuote)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(FetchTopTokens)
        .tool(DeployPumpFunToken)
        .tool(FetchTokenMetadata)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .tool(AnalyzeRisk)
        .tool(FetchPriceActionAnalysis)
        .tool(Think)
        .tool(AnalyzeHolderDistribution)
        .tool(AnalyzeSentiment)
        .tool(GetCurrentTime)
        .tool(SearchWeb)
        .tool(ViewImage)
        .tool(AnalyzePageContent);

    if features.autonomous {
        agent = agent.tool(Swap).tool(CreateAdvancedOrder);
    }

    agent.build()
}
