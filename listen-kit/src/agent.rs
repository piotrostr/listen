use crate::data::evm_fallback_tools::{
    FetchPriceActionAnalysisEvm, FetchTopTokensByCategory,
    FetchTopTokensByChainId,
};
use crate::evm::tools::{GetErc20Balance, GetEthBalance};
use crate::solana::tools::{
    DeployPumpFunToken, GetCurrentTime, GetSolBalance, GetSplTokenBalance,
};

use crate::agents::listen::create_deep_research_agent_openrouter;
use crate::agents::research::ViewImage;
use crate::common::{openrouter_agent_builder, OpenRouterAgent};
use crate::cross_chain::tools::{GetQuote, Swap};
use crate::data::{
    AnalyzePageContent, FetchPriceActionAnalysis, FetchTopTokens, FetchXPost,
    GetToken, ResearchXProfile, SearchTweets, SearchWeb,
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
// - gas sponsoring
// For Solana candlesticks, return the timestamp field as ISO date string
// TODO
// - Set up Sentry and grab any issue with the tool calls straight up (set up pager ideally)
// - It might be sound to include tool descriptions, which would require
//   extending the macro and a bit of migration but in the end might be a good
//   idea if model struggles with certain params
// - Use firecrawl instead of Exa (tight rate-limit and not good at "scrapes", like potential t.co/ redirects)

pub fn equip_with_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
        .tool(GetToken)
        .tool(GetQuote)
        .tool(GetSolBalance)
        .tool(GetSplTokenBalance)
        .tool(SearchOnDexScreener)
        .tool(FetchTopTokens)
        .tool(DeployPumpFunToken)
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

pub fn equip_with_evm_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
        .tool(GetEthBalance)
        .tool(GetErc20Balance)
        .tool(FetchPriceActionAnalysisEvm)
        .tool(FetchTopTokensByChainId)
        .tool(FetchTopTokensByCategory)
}

pub fn equip_with_worldchain_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
        .tool(Think)
        .tool(FetchTopTokensByChainId)
        .tool(GetQuote)
        .tool(GetEthBalance)
        .tool(GetErc20Balance)
        .tool(ResearchXProfile)
        .tool(FetchXPost)
        .tool(SearchTweets)
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
    if features.worldchain {
        return create_worldchain_agent(preamble);
    }

    let preamble = preamble.unwrap_or("".to_string());

    if features.deep_research {
        return create_deep_research_agent_openrouter(locale, None);
    }

    let mut agent = equip_with_evm_tools(equip_with_tools(
        openrouter_agent_builder(None),
    ))
    .preamble(&preamble);

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}

pub fn create_worldchain_agent(preamble: Option<String>) -> OpenRouterAgent {
    let preamble = preamble.unwrap_or("".to_string());

    let model = None;
    // let model = Some("google/gemini-2.5-flash-preview".to_string());

    let agent = equip_with_worldchain_tools(openrouter_agent_builder(model))
        .preamble(&preamble);

    agent.build()
}
