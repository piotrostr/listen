use anyhow::Result;
use futures::StreamExt;
use listen_memory::memory_system::MemorySystem;
use rig::completion::AssistantContent;
use rig::completion::Message;
use rig::message::{ToolResultContent, UserContent};
use rig::streaming::StreamingChoice;
use rig::OneOrMany;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use crate::memory::inject_memories;
use crate::memory::remember_tool_output;
use crate::reasoning_loop::Model;
use crate::reasoning_loop::SimpleToolResult;

use super::{ReasoningLoop, StreamResponse};

impl ReasoningLoop {
    pub async fn stream_generic(
        &self,
        model: Model,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
        memory_system: Option<Arc<MemorySystem>>,
    ) -> Result<Vec<Message>> {
        let mut current_messages = messages.clone();
        let stdout = self.stdout;

        // Start with the user's original prompt
        let mut next_input = Message::user(prompt.clone());
        let mut is_first_iteration = true;

        'outer: loop {
            let mut current_response = String::new();

            let _prompt = if is_first_iteration {
                let memory_system = memory_system.clone();
                if let Some(memory_system) = memory_system {
                    Message::user(
                        inject_memories(
                            memory_system.clone(),
                            prompt.clone(),
                        )
                        .await?,
                    )
                } else {
                    Message::user(prompt.clone())
                }
            } else {
                next_input.clone()
            };

            // Stream using the next input (original prompt or tool result)
            let mut stream = match model
                .stream_completion(_prompt.clone(), current_messages.clone())
                .await
            {
                Ok(stream) => stream,
                Err(e) => {
                    tracing::error!("Error: failed to stream chat: {}", e);
                    return Err(anyhow::anyhow!(
                        "failed to stream chat: {}",
                        e
                    ));
                }
            };

            // Only add the original user prompt to history on the first iteration
            if is_first_iteration {
                current_messages.push(Message::User {
                    content: OneOrMany::one(UserContent::text(
                        prompt.clone(),
                    )),
                });
                is_first_iteration = false;
            } else {
                // For subsequent iterations, add the tool result message to history
                current_messages.push(next_input.clone());
            }

            while let Some(chunk) = stream.next().await {
                match chunk? {
                    StreamingChoice::ParToolCall(tool_calls) => {
                        // Add the assistant's response up to this point with the tool calls
                        if !current_response.is_empty() {
                            current_messages.push(Message::Assistant {
                                content: OneOrMany::one(
                                    AssistantContent::text(
                                        current_response.clone(),
                                    ),
                                ),
                            });
                            current_response.clear();
                        }

                        let mut assistant_contents =
                            vec![None; tool_calls.len()];

                        for (index, tool_call) in tool_calls.iter() {
                            assistant_contents[*index] =
                                Some(AssistantContent::tool_call(
                                    tool_call.id.clone(),
                                    tool_call.function.name.clone(),
                                    tool_call.function.arguments.clone(),
                                ));
                        }

                        let assistant_contents: Vec<_> = assistant_contents
                            .into_iter()
                            .flatten()
                            .collect();

                        current_messages.push(Message::Assistant {
                            content: OneOrMany::many(assistant_contents)?,
                        });

                        if let Some(tx) = &tx {
                            // Convert HashMap to Vec<ToolCall>, sorted by index
                            let mut indexed_calls: Vec<_> =
                                tool_calls.iter().collect();
                            indexed_calls.sort_by_key(|(index, _)| **index);
                            let sorted_tool_calls: Vec<_> = indexed_calls
                                .into_iter()
                                .map(|(_, call)| call.clone())
                                .collect();

                            tx.send(StreamResponse::ParToolCall {
                                tool_calls: sorted_tool_calls,
                            })
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "failed to send tool call: {}",
                                    e
                                )
                            })?;
                        }

                        // Run all tool calls in parallel
                        let tasks =
                            tool_calls.iter().map(|(index, tool_call)| {
                                let model = model.clone();
                                let name = tool_call.function.name.clone();
                                let params =
                                    tool_call.function.arguments.to_string();
                                let id = tool_call.id.clone();

                                async move {
                                    let result = model
                                        .call_tool(
                                            name.clone(),
                                            params.clone(),
                                        )
                                        .await;
                                    let result_str = match &result {
                                        Ok(content) => content.to_string(),
                                        Err(err) => err.to_string(),
                                    };

                                    SimpleToolResult {
                                        index: *index,
                                        id,
                                        name,
                                        params,
                                        result: result_str,
                                    }
                                }
                            });

                        // Wait for all tool calls to complete
                        let mut results =
                            futures::future::join_all(tasks).await;
                        results.sort_by_key(|tool_result| tool_result.index);

                        if let Some(memory_system) = memory_system.clone() {
                            let results = results.clone();
                            for result in results {
                                let memory_system = memory_system.clone();
                                tokio::spawn(async move {
                                    match remember_tool_output(
                                        memory_system,
                                        result.name.clone(),
                                        result.params.clone(),
                                        result.result.clone(),
                                    )
                                    .await
                                    {
                                        Ok(_) => {}
                                        Err(e) => {
                                            tracing::error!(
                                                "Error: failed to remember tool output ({}): {}",
                                                result.name, e
                                            );
                                        }
                                    }
                                });
                            }
                        }

                        // Create a single message with all tool results
                        next_input = Message::User {
                            content: OneOrMany::many(
                                results
                                    .iter()
                                    .map(|tool_result| {
                                        UserContent::tool_result(
                                            tool_result.id.clone(),
                                            OneOrMany::one(
                                                ToolResultContent::text(
                                                    tool_result
                                                        .result
                                                        .clone(),
                                                ),
                                            ),
                                        )
                                    })
                                    .collect::<Vec<UserContent>>(),
                            )?,
                        };

                        if let Some(tx) = &tx {
                            tx.send(StreamResponse::ParToolResult {
                                tool_results: results,
                            })
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "failed to send tool result: {}",
                                    e
                                )
                            })?;
                        }

                        continue 'outer;
                    }
                    StreamingChoice::Message(text) => {
                        if stdout {
                            print!("{}", text);
                            std::io::stdout().flush()?;
                        } else if let Some(tx) = &tx {
                            tx.send(StreamResponse::Message(text.clone()))
                                .await
                                .map_err(|e| {
                                    anyhow::anyhow!(
                                        "failed to send message: {}",
                                        e
                                    )
                                })?;
                        }
                        current_response.push_str(&text);
                    }
                    StreamingChoice::ToolCall(name, tool_id, params) => {
                        // Add the assistant's response up to this point with the tool call
                        if !current_response.is_empty() {
                            current_messages.push(Message::Assistant {
                                content: OneOrMany::one(
                                    AssistantContent::text(
                                        current_response.clone(),
                                    ),
                                ),
                            });
                            current_response.clear();
                        }

                        // Add the tool use message from the assistant
                        current_messages.push(Message::Assistant {
                            content: OneOrMany::one(
                                AssistantContent::tool_call(
                                    tool_id.clone(),
                                    name.clone(),
                                    params.clone(),
                                ),
                            ),
                        });

                        if let Some(tx) = &tx {
                            tx.send(StreamResponse::ToolCall {
                                id: tool_id.clone(),
                                name: name.clone(),
                                params: params.to_string(),
                            })
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "failed to send tool call: {}",
                                    e
                                )
                            })?;
                        }

                        // Call the tool and get result
                        let result = model
                            .call_tool(name.to_string(), params.to_string())
                            .await;

                        if stdout {
                            print!("Tool result: {:?}\n", result);
                        }

                        // Create the tool result message to use directly as next input
                        let result_str = match &result {
                            Ok(content) => content.to_string(),
                            Err(err) => err.to_string(),
                        };

                        // Create the tool result message to be used as the next input
                        next_input = Message::User {
                            content: OneOrMany::one(
                                UserContent::tool_result(
                                    tool_id.clone(),
                                    OneOrMany::one(ToolResultContent::text(
                                        result_str.clone(),
                                    )),
                                ),
                            ),
                        };

                        if let Some(memory_system) = memory_system.clone() {
                            let name = name.clone();
                            let params = params.to_string();
                            let result_str = result_str.clone();
                            tokio::spawn(async move {
                                match remember_tool_output(
                                    memory_system,
                                    name.clone(),
                                    params,
                                    result_str,
                                )
                                .await
                                {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!(
                                            "Error: failed to remember tool output ({}): {}",
                                            name,
                                            e
                                        );
                                    }
                                }
                            });
                        }

                        if let Some(tx) = &tx {
                            tx.send(StreamResponse::ToolResult {
                                id: tool_id,
                                name,
                                result: result_str,
                            })
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "failed to send tool call: {}",
                                    e
                                )
                            })?;
                        }

                        continue 'outer;
                    }
                }
            }

            // Add any remaining response to messages
            if !current_response.is_empty() {
                current_messages.push(Message::Assistant {
                    content: OneOrMany::one(AssistantContent::text(
                        current_response,
                    )),
                });
            }

            // If we get here, there were no tool calls in this iteration
            break;
        }

        Ok(current_messages)
    }
}
