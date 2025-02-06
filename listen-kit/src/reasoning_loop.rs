use anyhow::Result;
use core::panic;
use futures_util::StreamExt;
use rig::agent::Agent;
use rig::completion::Message;
use rig::providers::anthropic::completion::CompletionModel;
use rig::streaming::{StreamingChat, StreamingChoice};
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub enum LoopResponse {
    Message(String),
    ToolCall { name: String, result: String },
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
        messages: Vec<Message>,
        tx: Option<Sender<LoopResponse>>,
    ) -> Result<Vec<Message>> {
        if tx.is_none() && !self.stdout {
            panic!("enable stdout or provide tx channel");
        }

        let mut current_messages = messages;

        'outer: loop {
            println!("current_messages: {:?}", current_messages);
            let mut stream =
                self.agent.stream_chat("", current_messages.clone()).await?;
            let mut current_response = String::new();

            while let Some(chunk) = stream.next().await {
                match chunk? {
                    StreamingChoice::Message(text) => {
                        if self.stdout {
                            print!("{}", text);
                            std::io::stdout().flush()?;
                        } else if let Some(tx) = &tx {
                            tx.send(LoopResponse::Message(text.clone()))
                                .await?;
                        }
                        current_response.push_str(&text);
                    }
                    StreamingChoice::ToolCall(name, tool_id, params) => {
                        let result = self
                            .agent
                            .tools
                            .call(&name, params.to_string())
                            .await;

                        if self.stdout {
                            println!("Tool result: {:?}", result);
                        }

                        // Add the assistant's response up to this point
                        if !current_response.is_empty() {
                            current_messages.push(Message {
                                role: "assistant".to_string(),
                                content: current_response.clone(),
                            });
                            current_response.clear();
                        }

                        // Add the tool result as a user message with proper structure
                        current_messages.push(Message {
                            role: "user".to_string(),
                            content: match &result {
                                Ok(content) => format!(
                                    "{{\"type\": \"tool_result\", \"tool_use_id\": \"{}\", \"content\": \"{}\"}}",
                                    tool_id, content
                                ),
                                Err(err) => format!(
                                    "{{\"type\": \"tool_result\", \"tool_use_id\": \"{}\", \"content\": \"{}\", \"is_error\": true}}",
                                    tool_id, err.to_string()
                                ),
                            },
                        });

                        if let Some(tx) = &tx {
                            tx.send(LoopResponse::ToolCall {
                                name,
                                result: match &result {
                                    Ok(content) => content.to_string(),
                                    Err(err) => err.to_string(),
                                },
                            })
                            .await?;
                        }

                        // Continue the outer loop with updated messages
                        continue 'outer;
                    }
                }
            }

            // Add any remaining response to messages
            if !current_response.is_empty() {
                current_messages.push(Message {
                    role: "assistant".to_string(),
                    content: current_response,
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
