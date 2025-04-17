use crate::evm::tools::{GetErc20Balance, GetEthBalance};
use crate::solana::tools::{
    DeployPumpFunToken, GetCurrentTime, GetSolBalance, GetSplTokenBalance,
};

use crate::agents::listen::create_deep_research_agent_openrouter;
use crate::agents::research::ViewImage;
use crate::common::{openrouter_agent_builder, OpenRouterAgent};
use crate::cross_chain::tools::{GetQuote, Swap};
use crate::data::{
    AnalyzePageContent, FetchPriceActionAnalysis, FetchTokenMetadata,
    FetchTopTokens, FetchXPost, ResearchXProfile, SearchTweets, SearchWeb,
};
use crate::dexscreener::tools::SearchOnDexScreener;
use crate::faster100x::AnalyzeHolderDistribution;
use crate::lunarcrush::AnalyzeSentiment;
use crate::solana::tools::AnalyzeRisk;
use crate::think::Think;

use rig::agent::AgentBuilder;
use rig::streaming::StreamingCompletionModel;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Features {
    pub autonomous: bool,
    pub deep_research: bool,
    #[serde(default)]
    pub memory: bool,
}

pub fn model_to_versioned_model(model_type: String) -> String {
    match model_type.as_str() {
        // FIXME implement proper rate lims on claude
        "claude" => "deepseek/deepseek-chat-v3-0324".to_string(), // "anthropic/claude-3.7-sonnet".to_string(),
        "gemini" => "google/gemini-2.0-flash-001".to_string(),
        "deepseek" => "deepseek/deepseek-chat-v3-0324".to_string(),
        "openai" => "openai/gpt-4o-mini".to_string(),
        "llama" => "meta-llama/llama-4-maverick".to_string(),
        _ => "google/gemini-2.0-flash-001".to_string(),
    }
}

// TODO
// newly added tools that require testing:
// - GetErc20Balance
// - GetEthBalance
// - GetQuote
// - Swap (direct) - changed to allow any token on any chain

pub fn equip_with_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
        .tool(GetQuote)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(GetEthBalance)
        .tool(GetErc20Balance)
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
        .tool(AnalyzePageContent)
}

pub fn equip_with_autonomous_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder.tool(Swap) // .tool(CreateAdvancedOrder)
}

// TODO ensure that the reserach trader agent has evm tools too
pub fn create_listen_agent(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> OpenRouterAgent {
    let preamble = preamble.unwrap_or("".to_string());
    let mut agent =
        equip_with_tools(openrouter_agent_builder(None)).preamble(&preamble);

    if features.deep_research {
        return create_deep_research_agent_openrouter(locale, None);
    }

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}
