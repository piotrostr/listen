use crate::agent::{equip_with_autonomous_tools, equip_with_tools, Features};
use crate::agents::listen::{
    create_deep_research_agent_claude, create_deep_research_agent_deepseek,
    create_deep_research_agent_gemini, create_deep_research_agent_openai,
    create_deep_research_agent_openrouter,
};
use crate::common::{
    claude_agent_builder, deepseek_agent_builder, gemini_agent_builder,
    openai_agent_builder, openrouter_agent_builder, ClaudeAgent,
    DeepSeekAgent, GeminiAgent, OpenAIAgent, OpenRouterAgent,
};

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

pub fn create_solana_agent_openrouter(
    preamble: Option<String>,
    features: Features,
    locale: String,
    model: Option<String>,
) -> OpenRouterAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

    if features.deep_research {
        return create_deep_research_agent_openrouter(locale, model);
    }

    let mut agent =
        equip_with_tools(openrouter_agent_builder(model)).preamble(&preamble);

    if features.autonomous {
        agent = equip_with_autonomous_tools(agent);
    }

    agent.build()
}

pub fn create_solana_agent_claude(
    preamble: Option<String>,
    features: Features,
    locale: String,
) -> ClaudeAgent {
    let preamble =
        preamble.unwrap_or("you are a solana trading agent".to_string());

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
