use rig::{agent::AgentBuilder, streaming::StreamingCompletionModel};

use crate::{
    agents::{
        chart::DelegateToChartAgent, research::DelegateToResearchAgent,
        trader::DelegateToTraderAgent,
    },
    common::{
        claude_agent_builder, deepseek_agent_builder, gemini_agent_builder,
        openai_agent_builder, openrouter_agent_builder, ClaudeAgent,
        DeepSeekAgent, GeminiAgent, OpenAIAgent, OpenRouterAgent,
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

const PREAMBLE_EN: &str = r#"
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

Keep investigating until you have explored all relevant angles.
Always use English."#;

const PREAMBLE_ZH: &str = r#"
你是一个规划代理，一个协调任务委托给专门代理的协调者。
你的目标是深入挖掘每个主题，通过：
1. 将复杂查询分解为更小、更集中的问题
2. 将每个问题委托给适当的代理
3. 分析他们的响应以识别需要更深入调查的差距或领域
4. 继续委托跟进问题，直到你获得全面见解

总是进行多个工具调用以构建完整图景。不要满足于表面信息。
对于每个任务，提供一系列提示，逐步深入主题。

格式化你的调查计划如下：
1. 初始问题：[委托给适当的代理]
2. 基于响应的跟进问题

请使用中文。"#;

pub fn equip_with_swarm_leader_tools<M: StreamingCompletionModel>(
    agent: AgentBuilder<M>,
) -> AgentBuilder<M> {
    agent
        .tool(FetchTokenMetadata)
        .tool(SearchOnDexScreener)
        .tool(DelegateToResearchAgent)
        .tool(DelegateToTraderAgent)
        .tool(DelegateToChartAgent)
        .tool(Think)
        .tool(GetCurrentTime)
        .tool(FetchTopTokens)
}

pub fn create_deep_research_agent_claude(locale: String) -> ClaudeAgent {
    equip_with_swarm_leader_tools(claude_agent_builder())
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .build()
}

pub fn create_deep_research_agent_gemini(locale: String) -> GeminiAgent {
    equip_with_swarm_leader_tools(gemini_agent_builder())
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .build()
}

pub fn create_deep_research_agent_deepseek(locale: String) -> DeepSeekAgent {
    equip_with_swarm_leader_tools(deepseek_agent_builder())
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .build()
}

pub fn create_deep_research_agent_openai(locale: String) -> OpenAIAgent {
    equip_with_swarm_leader_tools(openai_agent_builder())
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .build()
}

pub fn create_deep_research_agent_openrouter(
    locale: String,
    model: Option<String>,
) -> OpenRouterAgent {
    equip_with_swarm_leader_tools(openrouter_agent_builder(model))
        .preamble(if locale == "zh" {
            PREAMBLE_ZH
        } else {
            PREAMBLE_EN
        })
        .build()
}
