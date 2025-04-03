use anyhow::Result;
use futures::StreamExt;
use rig::completion::AssistantContent;
use rig::completion::Message;
use rig::message::{ToolResultContent, UserContent};
use rig::streaming::StreamingChoice;
use rig::streaming::StreamingCompletion;
use rig::OneOrMany;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use crate::common::DeepSeekAgent;

use super::{ReasoningLoop, StreamResponse};

impl ReasoningLoop {
    pub async fn stream_openai(
        &self,
        agent: &Arc<DeepSeekAgent>,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
    ) -> Result<Vec<Message>> {
        let mut current_messages = messages.clone();
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

                        let result_str = match &result {
                            Ok(content) => content.to_string(),
                            Err(err) => err.to_string(),
                        };

                        // Create the tool result message to be used as the next input
                        next_input = Message::User {
                            content: OneOrMany::one(
                                UserContent::tool_result(
                                    name.clone(),
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
