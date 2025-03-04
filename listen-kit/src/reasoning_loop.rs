use anyhow::Result;
use futures::StreamExt;
use rig::agent::Agent;
use rig::completion::AssistantContent;
use rig::completion::Message;
use rig::message::{ToolResultContent, UserContent};
use rig::providers::anthropic::completion::CompletionModel;
use rig::streaming::{StreamingChat, StreamingChoice};
use rig::OneOrMany;
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum StreamResponse {
    Message(String),
    ToolCall {
        id: String,
        name: String,
        params: String,
    },
    ToolResult {
        id: String,
        name: String,
        result: String,
    },
    Error(String),
}

pub struct ReasoningLoop {
    agent: Arc<Agent<CompletionModel>>,
    stdout: bool,
}

impl ReasoningLoop {
    pub fn new(agent: Arc<Agent<CompletionModel>>) -> Self {
        Self {
            agent,
            stdout: true,
        }
    }

    pub async fn stream(
        &self,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
    ) -> Result<Vec<Message>> {
        if tx.is_none() && !self.stdout {
            panic!("enable stdout or provide tx channel");
        }

        let mut current_messages = messages.clone();
        let agent = self.agent.clone();
        let stdout = self.stdout;

        // For first iteration, use the original prompt.
        // For subsequent iterations, use an empty prompt since we already have the conversation history.
        let mut is_first_iteration = true;

        'outer: loop {
            let mut current_response = String::new();

            // Use the original prompt only for the first iteration
            let current_prompt = if is_first_iteration {
                prompt.clone()
            } else {
                // Minimal, neutral prompt for subsequent iterations that won't trigger specific behaviors
                // this is not going to be added to the conversation history
                // TODO there might be a better way to handle this
                "Continue the conversation.".to_string()
            };

            let mut stream = match agent
                .stream_chat(&current_prompt, current_messages.clone())
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

            if is_first_iteration {
                current_messages.push(Message::User {
                    content: OneOrMany::one(UserContent::text(
                        prompt.clone(),
                    )),
                });
                is_first_iteration = false;
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
                        let result = self
                            .agent
                            .tools
                            .call(&name, params.to_string())
                            .await;

                        if stdout {
                            println!("Tool result: {:?}", result);
                        }

                        // Add the tool result as a user message
                        current_messages.push(Message::User {
                            content: OneOrMany::one(
                                UserContent::tool_result(
                                    tool_id.clone(),
                                    OneOrMany::one(ToolResultContent::text(
                                        match &result {
                                            Ok(content) => {
                                                content.to_string()
                                            }
                                            Err(err) => err.to_string(),
                                        },
                                    )),
                                ),
                            ),
                        });

                        if let Some(tx) = &tx {
                            tx.send(StreamResponse::ToolResult {
                                id: tool_id,
                                name,
                                result: match &result {
                                    Ok(content) => content.to_string(),
                                    Err(err) => err.to_string(),
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

    pub fn with_stdout(mut self, enabled: bool) -> Self {
        self.stdout = enabled;
        self
    }
}
