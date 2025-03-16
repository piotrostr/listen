use super::tools::{
    DeployPumpFunToken, GetQuote, GetSolBalance, GetSplTokenBalance, Swap,
};
use crate::common::{claude_agent_builder, PREAMBLE_COMMON};
use crate::data::{
    FetchPriceActionAnalysis, FetchTokenMetadata, FetchTopTokens, FetchXPost,
    ResearchXProfile, SearchTweets,
};
use crate::dexscreener::tools::SearchOnDexScreener;
use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Features {
    pub autonomous: bool,
}

pub async fn create_solana_agent(
    preamble: Option<String>,
    features: Features,
) -> Result<Agent<AnthropicCompletionModel>> {
    let preamble = preamble.unwrap_or(format!(
        "{} {}",
        "you are a solana trading agent that can also interact with pump.fun;",
        PREAMBLE_COMMON
    ));

    let mut agent = claude_agent_builder()
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
        .tool(FetchPriceActionAnalysis);

    if features.autonomous {
        agent = agent.tool(Swap);
    }

    Ok(agent.build())
}
