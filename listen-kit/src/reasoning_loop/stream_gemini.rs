use anyhow::Result;
use futures::StreamExt;
use rig::completion::AssistantContent;
use rig::completion::Message;
use rig::message::ToolResult;
use rig::message::{ToolResultContent, UserContent};
use rig::streaming::StreamingChoice;
use rig::streaming::StreamingCompletion;
use rig::OneOrMany;
use serde_json::json;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use crate::common::GeminiAgent;

use super::{ReasoningLoop, StreamResponse};

impl ReasoningLoop {
    pub fn geminify_chat_history(messages: Vec<Message>) -> Vec<Message> {
        messages
            .into_iter()
            .map(|msg| {
                match msg {
                    Message::User { content } => {
                        if let UserContent::ToolResult(tool_result) =
                            content.first()
                        {
                            // Wrap tool result content in the expected format
                            let result_content = tool_result
                                .content
                                .into_iter()
                                .next()
                                .map(|c| match c {
                                    ToolResultContent::Text(text) => {
                                        text.text
                                    }
                                    _ => panic!(
                                        "Tool result content is not a text"
                                    ),
                                })
                                .unwrap_or_default();

                            // Check if it contains "error" to determine format
                            let wrapped_content = if result_content
                                .to_lowercase()
                                .contains("error")
                            {
                                json!({"error": result_content})
                            } else {
                                json!({"result": result_content})
                            }
                            .to_string();

                            Message::User {
                                content: OneOrMany::one(
                                    UserContent::ToolResult(ToolResult {
                                        id: tool_result.id,
                                        content: OneOrMany::one(
                                            ToolResultContent::text(
                                                wrapped_content,
                                            ),
                                        ),
                                    }),
                                ),
                            }
                        } else {
                            // Not a tool result, leave as-is
                            Message::User { content }
                        }
                    }
                    // Leave assistant messages unchanged
                    Message::Assistant { content } => {
                        Message::Assistant { content }
                    }
                }
            })
            .collect()
    }

    pub async fn stream_gemini(
        &self,
        agent: &Arc<GeminiAgent>,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
    ) -> Result<Vec<Message>> {
        let mut current_messages =
            Self::geminify_chat_history(messages.clone());
        let agent = agent.clone();
        let stdout = self.stdout;

        // Start with the user's original prompt
        let mut next_input = Message::user(prompt.clone());
        let mut is_first_iteration = true;

        'outer: loop {
            let mut current_response = String::new();

            // Stream using the next input (original prompt or tool result)
            let mut stream = match agent
                .stream_completion(
                    next_input.clone(),
                    current_messages.clone(),
                )
                .await?
                .stream()
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
                    StreamingChoice::ParToolCall(_tool_call) => todo!(),
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
                        let result =
                            agent.tools.call(&name, params.to_string()).await;

                        if stdout {
                            print!("Tool result: {:?}\n", result);
                        }

                        // Create the tool result message to use directly as next input
                        let mut is_err = false;
                        let result_str = match &result {
                            Ok(content) => serde_json::json!({"result": content.to_string()}).to_string(),
                            Err(err) => {
                                is_err = true;
                                serde_json::json!({"error": err.to_string()})
                                    .to_string()
                            }
                        };

                        // Create the tool result message to be used as the next input
                        next_input = Message::User {
                            content: OneOrMany::one(
                                UserContent::tool_result(
                                    name.clone(), // TODO gemini doesn't have tool IDs, not sure if `name` is the param
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
                                result: match is_err {
                                    true => serde_json::from_str::<
                                        serde_json::Value,
                                    >(
                                        &result_str
                                    )?["error"]
                                        .as_str()
                                        .unwrap_or_default()
                                        .to_string(),
                                    false => serde_json::from_str::<
                                        serde_json::Value,
                                    >(
                                        &result_str
                                    )?["result"]
                                        .as_str()
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| {
                                            // If it's not a string, serialize the value directly
                                            serde_json::to_string(
                                                &serde_json::from_str::<
                                                    serde_json::Value,
                                                >(
                                                    &result_str
                                                )
                                                .unwrap()["result"],
                                            )
                                            .unwrap_or_default()
                                        }),
                                },
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
