use anyhow::Result;
use listen_memory::{
    mem0::{AddMemoryConfig, Mem0, Message as Mem0Message, SearchFilters},
    memory_system::MemorySystem,
};
use std::sync::Arc;

pub async fn inject_memories(
    prompt: String,
    user_id: Option<String>,
) -> Result<String> {
    let mem0 = Mem0::default();

    let filters = if let Some(user_id) = user_id {
        Some(SearchFilters {
            user_id: Some(user_id),
            agent_id: None,
            run_id: None,
        })
    } else {
        None
    };

    let memories = mem0.search_memories(prompt.clone(), filters).await?;

    let injected_prompt = format!(
        "<user-prompt>{}</user-prompt><relevant-memories>{}</relevant-memories>",
        prompt, serde_json::to_string(&memories)?
    );
    println!("injected_prompt: {}", injected_prompt);
    Ok(injected_prompt)
}

pub async fn _inject_memories(
    memory_system: Arc<MemorySystem>,
    prompt: String,
) -> Result<String> {
    let memories = memory_system
        .find_related_memories(prompt.clone(), 5)
        .await?;
    let memory = memory_system
        .summarize_relevant_memories(memories, prompt.clone())
        .await?;
    let injected_prompt = format!(
        "<user-prompt>{}</user-prompt><relevant-memories>{}</relevant-memories>",
        prompt, memory
    );
    println!("injected_prompt: {}", injected_prompt);
    Ok(injected_prompt)
}

const TOOLS_WORTH_REMEMBERING: [&str; 9] = [
    "fetch_token_metadata",
    "research_x_profile",
    "fetch_x_post",
    "search_tweets",
    "fetch_price_action_analysis",
    "analyze_sentiment",
    "search_web",
    "analyze_page_content",
    "view_image",
];

pub async fn _remember_tool_output(
    memory_system: Arc<MemorySystem>,
    tool_name: String,
    tool_params: String,
    tool_result: String,
) -> Result<()> {
    if !TOOLS_WORTH_REMEMBERING.contains(&tool_name.as_str()) {
        return Ok(());
    }

    let note = format!(
        "Result of tool call {} with params: {}\nResult: {}",
        tool_name, tool_params, tool_result
    );

    memory_system.add_note(note).await?;

    Ok(())
}

// TODO is there an opinionated way of going about Mem0 tool messages?
pub async fn remember_tool_output(
    tool_name: String,
    tool_params: String,
    tool_result: String,
) -> Result<()> {
    let mem0 = Mem0::default();
    let res = mem0
        .add_memory(
            vec![Mem0Message {
                role: "user".to_string(),
                content: format!(
                    "Result of tool call {} with params: {}: {}",
                    tool_name, tool_params, tool_result
                ),
            }],
            AddMemoryConfig::default(),
        )
        .await?;

    println!("mem0 result: {:?}", res);

    Ok(())
}
