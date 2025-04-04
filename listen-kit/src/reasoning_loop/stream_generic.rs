use anyhow::Result;
use futures::StreamExt;
use rig::completion::AssistantContent;
use rig::completion::Message;
use rig::message::{ToolResultContent, UserContent};
use rig::streaming::StreamingChoice;
use rig::OneOrMany;
use std::io::Write;
use tokio::sync::mpsc::Sender;

use crate::reasoning_loop::Model;

use super::{ReasoningLoop, StreamResponse};

impl ReasoningLoop {
    pub async fn stream_generic(
        &self,
        model: Model,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
    ) -> Result<Vec<Message>> {
        let mut current_messages = messages.clone();
        let stdout = self.stdout;

        // Start with the user's original prompt
        let mut next_input = Message::user(prompt.clone());
        let mut is_first_iteration = true;

        'outer: loop {
            let mut current_response = String::new();

            // Stream using the next input (original prompt or tool result)
            let mut stream = match model
                .stream_completion(
                    next_input.clone(),
                    current_messages.clone(),
                )
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

                        current_messages.push(Message::Assistant {
                            content: OneOrMany::many(
                                tool_calls
                                    .values()
                                    .map(|tool_call| {
                                        AssistantContent::tool_call(
                                            tool_call.id.clone(),
                                            tool_call.function.name.clone(),
                                            tool_call
                                                .function
                                                .arguments
                                                .clone(),
                                        )
                                    })
                                    .collect::<Vec<AssistantContent>>(),
                            )?,
                        });

                        if let Some(tx) = &tx {
                            tx.send(StreamResponse::ParToolCall {
                                tool_calls: tool_calls.clone(),
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
                        let tasks = tool_calls.values().map(|tool_call| {
                            let model = model.clone();
                            let name = tool_call.function.name.clone();
                            let params =
                                tool_call.function.arguments.to_string();
                            let id = tool_call.id.clone();

                            async move {
                                let result =
                                    model.call_tool(name, params).await;
                                let result_str = match &result {
                                    Ok(content) => content.to_string(),
                                    Err(err) => err.to_string(),
                                };
                                (id, result_str)
                            }
                        });

                        // Wait for all tool calls to complete
                        let results = futures::future::join_all(tasks).await;

                        // Create a single message with all tool results
                        next_input = Message::User {
                            content: OneOrMany::many(
                                results
                                    .iter()
                                    .map(|(id, result_str)| {
                                        UserContent::tool_result(
                                            id.clone(),
                                            OneOrMany::one(
                                                ToolResultContent::text(
                                                    result_str.clone(),
                                                ),
                                            ),
                                        )
                                    })
                                    .collect::<Vec<UserContent>>(),
                            )?,
                        };

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
