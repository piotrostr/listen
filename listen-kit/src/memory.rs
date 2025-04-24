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
    global_memory: Option<Arc<GraphMemory>>,
    prompt: String,
    user_id: String,
) -> Result<String> {
    let (global_memories, user_specific_memories) = match global_memory {
        Some(mem) => {
            // Run both searches in parallel when we have global memory
            let mem0 = Mem0::default();
            let op = mem0.search_memories(prompt.clone(), user_id.clone());
            let (global_mems, user_mems) = tokio::join!(
                mem.search(prompt.as_str(), Filters {}, Some(15)),
                op
            );

            let global_mems = match global_mems {
                Ok(mems) => Some(mems),
                Err(e) => {
                    tracing::error!("Failed to fetch global memories: {}", e);
                    None
                }
            };

            let user_mems = match user_mems {
                Ok(mems) => mems,
                Err(e) => {
                    tracing::error!("Failed to fetch user memories: {}", e);
                    mem0.empty_search_result()
                }
            };

            (global_mems, user_mems)
        }
        None => {
            // Just fetch user memories when no global memory
            let mem0 = Mem0::default();
            let user_mems = match mem0
                .search_memories(prompt.clone(), user_id)
                .await
            {
                Ok(mems) => mems,
                Err(e) => {
                    tracing::error!("Failed to fetch user memories: {}", e);
                    mem0.empty_search_result()
                }
            };
            (None, user_mems)
        }
    };

    // Create base prompt without memories if serialization fails
    let mut injected_prompt =
        format!("<USER PROMPT>{}</USER PROMPT>", prompt);

    // Try to add user specific memories
    match serde_json::to_string(
        &user_specific_memories
            .results
            .iter()
            .map(|m| m.distill())
            .collect::<Vec<_>>(),
    ) {
        Ok(user_mems_str) => {
            injected_prompt = format!(
                "{}\n<USER SPECIFIC MEMORIES>{}</USER SPECIFIC MEMORIES>",
                injected_prompt, user_mems_str
            );

            // Debug logging only if serialization succeeds
            if let Ok(pretty_str) = serde_json::to_string_pretty(
                &user_specific_memories
                    .results
                    .iter()
                    .map(|m| m.distill())
                    .collect::<Vec<_>>(),
            ) {
                println!("user_specific_memories: {}", pretty_str);
            }
        }
        Err(e) => {
            tracing::error!("Failed to serialize user memories: {}", e);
        }
    }

    // Add global memories if they exist and aren't empty
    if let Some(memories) = global_memories {
        if !memories.is_empty() {
            match serde_json::to_string(
                &memories.iter().map(|m| m.stringify()).collect::<Vec<_>>(),
            ) {
                Ok(global_mems_str) => {
                    injected_prompt = format!(
                        "{}\n<GLOBAL MEMORIES>{}</GLOBAL MEMORIES>",
                        injected_prompt, global_mems_str
                    );

                    // Debug logging only if serialization succeeds
                    if let Ok(pretty_str) =
                        serde_json::to_string_pretty(&memories)
                    {
                        println!("memories: {}", pretty_str);
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to serialize global memories: {}",
                        e
                    );
                }
            }
        }
    }

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
