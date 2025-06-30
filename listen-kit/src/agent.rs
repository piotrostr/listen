use crate::data::evm_fallback_tools::{
    FetchPriceActionAnalysisEvm, FetchTopTokensByCategory,
    FetchTopTokensByChainId,
};
use crate::evm::tools::{GetErc20Balance, GetEthBalance};
use crate::hype::equip_with_hype_tools;
use crate::solana::tools::{
    DeployPumpFunToken, GetCurrentTime, GetSolBalance, GetSplTokenBalance,
};

use crate::agents::listen::create_deep_research_agent_openrouter;
use crate::agents::research::ViewImage;
use crate::common::{openrouter_agent_builder, OpenRouterAgent};
use crate::cross_chain::tools::{GetQuote, Swap};
use crate::data::{
    AnalyzePageContent, FetchPriceActionAnalysis, FetchTopTokens, FetchXPost,
    GetToken, GetTokenBalance, ResearchXProfile, SearchTweets, SearchWeb,
};
use crate::dexscreener::tools::SearchOnDexScreener;
use crate::faster100x::AnalyzeHolderDistribution;
use crate::lunarcrush::AnalyzeSentiment;
use crate::solana::tools::AnalyzeRisk;
use crate::think::Think;

use rig::agent::AgentBuilder;
use rig::streaming::StreamingCompletionModel;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Features {
    pub autonomous: bool,
    pub deep_research: bool,
    #[serde(default)]
    pub memory: bool,
    #[serde(default)]
    pub worldchain: bool,
    #[serde(default)]
    pub hyperliquid: bool,
}

pub fn model_to_versioned_model(model_type: String) -> String {
    match model_type.as_str() {
        // FIXME implement proper rate lims on claude
        "claude" => "anthropic/claude-3.5-sonnet".to_string(), // "anthropic/claude-3.7-sonnet".to_string(),
        "gemini" => "google/gemini-2.0-flash-001".to_string(),
        "deepseek" => "deepseek/deepseek-chat-v3-0324".to_string(),
        "openai" => "openai/gpt-4o-mini".to_string(),
        "llama" => "meta-llama/llama-4-maverick".to_string(),
        _ => "google/gemini-2.0-flash-001".to_string(),
    }
}

// TODOs
// add the display for the unified get_token_balance tool (any token, currently 4 tools)

pub fn equip_with_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
        .tool(GetToken)
        .tool(GetQuote)
        .tool(GetTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(FetchTopTokens) // TODO use GT for this
        .tool(DeployPumpFunToken)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .tool(AnalyzeRisk) // TODO add evm equivalent
        .tool(FetchPriceActionAnalysis)
        .tool(Think)
        .tool(AnalyzeHolderDistribution)
        .tool(AnalyzeSentiment) // TODO possibly drop? lunarcrush-specific
        .tool(GetCurrentTime)
        .tool(SearchWeb)
        .tool(ViewImage)
        .tool(AnalyzePageContent)
}

pub fn equip_with_evm_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
        .tool(FetchPriceActionAnalysisEvm) // TODO unify
        .tool(FetchTopTokensByChainId) // TODO unify with solana FetchTopTokens
        .tool(FetchTopTokensByCategory)
}

pub fn equip_with_worldchain_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
        .tool(Think)
        .tool(FetchTopTokensByChainId)
        .tool(GetQuote)
        .tool(GetToken)
        .tool(GetEthBalance)
        .tool(GetErc20Balance)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
        .tool(SearchWeb)
        .tool(ViewImage)
        .tool(AnalyzePageContent)
        .tool(SearchOnDexScreener)
}

pub fn equip_with_autonomous_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder.tool(Swap) // .tool(CreateAdvancedOrder)
}

pub fn create_listen_agent(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> OpenRouterAgent {
    if features.worldchain {
        return create_worldchain_agent(preamble);
    }

    let preamble = preamble.unwrap_or("".to_string());

    if features.deep_research {
        return create_deep_research_agent_openrouter(locale, None);
    }
    let model = Some("google/gemini-2.5-flash-preview".to_string());

    let mut agent = equip_with_evm_tools(equip_with_tools(
        openrouter_agent_builder(model),
    ))
    .preamble(&preamble);

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    if features.hyperliquid {
        agent = equip_with_hype_tools(agent);
    }

    agent.build()
}

pub fn create_worldchain_agent(preamble: Option<String>) -> OpenRouterAgent {
    let preamble = preamble.unwrap_or("".to_string());
    let model = Some("google/gemini-2.5-flash-preview".to_string());

    let agent = equip_with_worldchain_tools(openrouter_agent_builder(model))
        .preamble(&preamble);

    agent.build()
}
