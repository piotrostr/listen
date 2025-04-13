use anyhow::Result;
use listen_memory::{
    graph::{Filters, GraphMemory},
    memory_system::MemorySystem,
};
use std::sync::Arc;

pub async fn inject_memories(
    prompt: String,
    _user_id: Option<String>,
) -> Result<String> {
    let memory = GraphMemory::from_env().await?; // TODO this might need to be shared across threads

    let memories =
        memory.search(prompt.as_str(), Filters {}, Some(15)).await?;

    let injected_prompt = format!(
        "<user-prompt>{}</user-prompt><relevant-memories>{}</relevant-memories>",
        prompt,
        serde_json::to_string(&memories.iter().map(|m| m.stringify()).collect::<Vec<_>>())?
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
    // "fetch_price_action_analysis", TODO this requires special treatment
    "analyze_sentiment",
    "search_web",
    "analyze_page_content",
    "view_image",
    "search_on_dex_screener",
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
    if !TOOLS_WORTH_REMEMBERING.contains(&tool_name.as_str()) {
        return Ok(());
    }

    let memory = GraphMemory::from_env().await?;
    let res = memory
        .add(
            &format!(
                "Result of tool call {} with params: {}: {}",
                tool_name, tool_params, tool_result
            ),
            Filters {},
        )
        .await?;

    println!("mem0 result: {:?}", res);

    Ok(())
}
