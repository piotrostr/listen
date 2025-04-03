use crate::{
    common::{claude_agent_builder, ClaudeAgent},
    cross_chain::tools::{ApproveToken, CheckApproval, GetQuote, Swap},
    data::{FetchPriceActionAnalysis, FetchTopTokens},
    dexscreener::tools::SearchOnDexScreener,
};

pub fn create_cross_chain_agent(preamble: Option<String>) -> ClaudeAgent {
    let preamble =
        preamble.unwrap_or("you are a cross-chain trading agent".to_string());
    let agent_builder = claude_agent_builder()
        .preamble(&preamble)
        .tool(SearchOnDexScreener)
        .tool(GetQuote)
        .tool(Swap)
        .tool(ApproveToken)
        .tool(CheckApproval)
        .tool(FetchPriceActionAnalysis)
        .tool(FetchTopTokens);
    agent_builder.build()
}
