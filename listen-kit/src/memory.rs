use anyhow::Result;
use listen_memory::{
    graph::{Filters, GraphMemory},
    mem0::{Mem0, Message},
};
use std::sync::Arc;

use crate::reasoning_loop::StreamResponse;

pub fn make_mem0_messages(
    responses: Vec<StreamResponse>,
    prompt: String,
) -> Vec<Message> {
    let mut messages = responses
        .iter()
        .filter_map(|response| {
            let assistant_response =
                if let StreamResponse::Message(message) = response {
                    Some(Message {
                        role: "assistant".to_string(),
                        content: message.clone(),
                    })
                } else {
                    None
                };

            assistant_response
        })
        .collect::<Vec<_>>();

    messages.push(Message {
        role: "user".to_string(),
        content: prompt,
    });

    messages.reverse();

    messages
}

pub async fn inject_memories(
    global_memory: Arc<GraphMemory>,
    prompt: String,
    user_id: String,
) -> Result<String> {
    let memories = global_memory
        .search(prompt.as_str(), Filters {}, Some(15))
        .await?;

    let user_specific_memories = Mem0::default()
        .search_memories(prompt.clone(), user_id)
        .await?;

    println!("memories: {}", serde_json::to_string_pretty(&memories)?);
    println!(
        "user_specific_memories: {}",
        serde_json::to_string_pretty(&user_specific_memories)?
    );

    let injected_prompt = format!(
        "<USER PROMPT>{}</USER PROMPT>
        <GLOBAL MEMORIES>{}</GLOBAL MEMORIES>
        <USER SPECIFIC MEMORIES>{}</USER SPECIFIC MEMORIES>
        ",
        prompt,
        serde_json::to_string(
            &memories.iter().map(|m| m.stringify()).collect::<Vec<_>>()
        )?,
        serde_json::to_string(
            &user_specific_memories
                .results
                .iter()
                .map(|m| m.stringify())
                .collect::<Vec<_>>()
        )?,
    );

    Ok(injected_prompt)
}

const TOOLS_WORTH_REMEMBERING: [&str; 9] = [
    "fetch_top_tokens",
    "fetch_token_metadata",
    "research_x_profile",
    "fetch_x_post",
    "search_tweets",
    // "fetch_price_action_analysis", TODO this requires special treatment
    // "analyze_sentiment", -- this yields a lot of raw keywords, highly sparse
    // "fetch_price_action_analysis", TODO this requires special treatment
    // "analyze_sentiment", -- this yields a lot of raw keywords, highly sparse
    "search_web",
    "analyze_page_content",
    "view_image",
    "search_on_dex_screener",
];

/// those are tools with sparse output, difficult to ingest in one go
const TOOLS_REQUIRING_DISTILLATION: [&str; 4] = [
    "fetch_token_metadata",
    "fetch_x_post",
    "search_on_dex_screener",
    "fetch_top_tokens",
];

pub async fn add_user_specific_memories(
    user_id: String,
    messages: Vec<Message>,
) -> Result<()> {
    let memories = Mem0::default().add_memory(messages, user_id).await?;
    println!("added user specific memories: {:?}", memories);
    Ok(())
}

pub async fn remember_tool_output(
    global_memory: Arc<GraphMemory>,
    tool_name: String,
    tool_params: String,
    tool_result: String,
) -> Result<()> {
    if !TOOLS_WORTH_REMEMBERING.contains(&tool_name.as_str()) {
        return Ok(());
    }
    let tool_result = if TOOLS_REQUIRING_DISTILLATION
        .contains(&tool_name.as_str())
    {
        listen_memory::graph::distiller::distill(tool_result.as_str()).await?
    } else {
        tool_result
    };

    let res = global_memory
        .add(
            &format!(
                "Result of tool call {} with params: {}: {}",
                tool_name, tool_params, tool_result
            ),
            Filters {},
        )
        .await?;

    // also dump to debug
    if std::env::var("DUMP").is_ok() {
        // write to file in ./tool_output_samples
        let file_path = "./tool_output_samples";
        std::fs::create_dir_all(file_path)?;
        let file_name = format!(
            "{}/{}-{}.json",
            file_path,
            chrono::Utc::now().timestamp(),
            tool_name
        );
        std::fs::write(
            file_name,
            serde_json::json!({
                "tool_name": tool_name,
                "tool_params": tool_params,
                "tool_result": tool_result
            })
            .to_string(),
        )?;
    }

    println!("graph memory result: {:?}", res);
    // also dump to debug
    if std::env::var("DUMP").is_ok() {
        // write to file in ./tool_output_samples
        let file_path = "./tool_output_samples";
        std::fs::create_dir_all(file_path)?;
        let file_name = format!(
            "{}/{}-{}.json",
            file_path,
            chrono::Utc::now().timestamp(),
            tool_name
        );
        std::fs::write(
            file_name,
            serde_json::json!({
                "tool_name": tool_name,
                "tool_params": tool_params,
                "tool_result": tool_result
            })
            .to_string(),
        )?;
    }

    println!("graph memory result: {:?}", res);

    Ok(())
}
