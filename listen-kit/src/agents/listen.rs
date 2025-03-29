use crate::{
    agents::{
        chart::DelegateToChartAgent,
        solana_trader::DelegateToSolanaTraderAgent, web::DelegateToWebAgent,
        x::DelegateToXAgent,
    },
    common::{
        claude_agent_builder, gemini_agent_builder, ClaudeAgent, GeminiAgent,
    },
    data::{FetchTokenMetadata, FetchTopTokens},
    dexscreener::tools::SearchOnDexScreener,
    solana::tools::GetCurrentTime,
    think::Think,
};

// maybe just put out all of the agents in the bridge and stream the outputs using channels, display this on the frontend based on { agent: { output: ..events } }?
// could be simple and deadly effective
pub struct ListenBridge {}

// Listen, as the swarm leader, plans out the task which is delegated to subsequent agents
// it then can assess the outputs and evaluate as done or needs more information, or a retry

const PREAMBLE: &str = r#"
You are a planning agent, a coordinator that delegates tasks to specialized agents.
Your goal is to dig as deep as possible into each topic by:
1. Breaking down complex queries into smaller, focused questions
2. Delegating each question to appropriate agents
3. Analyzing their responses to identify gaps or areas needing deeper investigation
4. Continuing to delegate follow-up questions until you have comprehensive insights

Always make multiple tool calls to build a complete picture. Never be satisfied with surface-level information.
For each task, provide a series of prompts that progressively dig deeper into the topic.

Format your investigation plan like this:
1. Initial question: [delegate to appropriate agent]
2. Follow-up areas based on response
3. Deep-dive questions for each area

Keep investigating until you have explored all relevant angles."#;

pub fn create_listen_agent_claude() -> ClaudeAgent {
    claude_agent_builder()
        .tool(FetchTokenMetadata)
        .tool(SearchOnDexScreener)
        .tool(DelegateToXAgent)
        .tool(DelegateToWebAgent)
        .tool(DelegateToSolanaTraderAgent)
        .tool(DelegateToChartAgent)
        .tool(Think)
        .tool(GetCurrentTime)
        .tool(FetchTopTokens)
        .preamble(PREAMBLE)
        .build()
}

pub fn create_listen_agent_gemini() -> GeminiAgent {
    gemini_agent_builder()
        .tool(FetchTokenMetadata)
        .tool(SearchOnDexScreener)
        .tool(DelegateToXAgent)
        .tool(DelegateToWebAgent)
        .tool(DelegateToSolanaTraderAgent)
        .tool(DelegateToChartAgent)
        .tool(Think)
        .tool(GetCurrentTime)
        .tool(FetchTopTokens)
        .preamble(PREAMBLE)
        .build()
}
