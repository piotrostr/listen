use super::tools::{
    DeployPumpFunToken, GetCurrentTime, GetQuote, GetSolBalance,
    GetSplTokenBalance, Swap,
};
use crate::agents::listen::{
    create_deep_research_agent_claude, create_deep_research_agent_deepseek,
    create_deep_research_agent_gemini, create_deep_research_agent_openai,
    create_deep_research_agent_openrouter,
};
use crate::agents::research::ViewImage;
use crate::common::{
    claude_agent_builder, deepseek_agent_builder, gemini_agent_builder,
    openai_agent_builder, openrouter_agent_builder, ClaudeAgent,
    DeepSeekAgent, GeminiAgent, OpenAIAgent, OpenRouterAgent,
};
use crate::data::{
    AnalyzePageContent, FetchPriceActionAnalysis, FetchTokenMetadata,
    FetchTopTokens, FetchXPost, ResearchXProfile, SearchTweets, SearchWeb,
};
use crate::dexscreener::tools::SearchOnDexScreener;
use crate::faster100x::AnalyzeHolderDistribution;
use crate::lunarcrush::AnalyzeSentiment;
use crate::solana::advanced_orders::CreateAdvancedOrder;
use crate::solana::tools::AnalyzeRisk;
use crate::think::Think;

use rig::agent::AgentBuilder;
use rig::streaming::StreamingCompletionModel;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Features {
    pub autonomous: bool,
    pub deep_research: bool,
}

pub fn equip_with_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder
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
        .tool(AnalyzePageContent)
}

pub fn equip_with_autonomous_tools<M: StreamingCompletionModel>(
    agent_builder: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent_builder.tool(Swap).tool(CreateAdvancedOrder)
}

pub fn create_solana_agent_claude(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> ClaudeAgent {
    let preamble = preamble.unwrap_or(format!(
        "{}",
        "you are a solana trading agent that can also interact with pump.fun;"
    ));

    if features.deep_research {
        return create_deep_research_agent_claude(locale);
    }

    let mut agent =
        equip_with_tools(claude_agent_builder()).preamble(&preamble);

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}

pub fn create_solana_agent_gemini(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> GeminiAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    if features.deep_research {
        return create_deep_research_agent_gemini(locale);
    }

    let mut agent =
        equip_with_tools(gemini_agent_builder()).preamble(&preamble);

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}

pub fn create_solana_agent_deepseek(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> DeepSeekAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    if features.deep_research {
        return create_deep_research_agent_deepseek(locale);
    }

    let mut agent =
        equip_with_tools(deepseek_agent_builder()).preamble(&preamble);

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}

pub fn create_solana_agent_openai(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> OpenAIAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    if features.deep_research {
        return create_deep_research_agent_openai(locale);
    }

    let mut agent =
        equip_with_tools(openai_agent_builder()).preamble(&preamble);

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}

pub fn create_solana_agent_openrouter(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> OpenRouterAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    if features.deep_research {
        return create_deep_research_agent_openrouter(locale);
    }

    let mut agent =
        equip_with_tools(openrouter_agent_builder()).preamble(&preamble);

    if features.deep_research {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}
