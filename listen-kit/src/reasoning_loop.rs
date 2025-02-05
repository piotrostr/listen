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

        loop {
            let mut stream =
                self.agent.stream_chat("", current_messages.clone()).await?;
            let mut current_response = String::new();
            let mut tool_results = Vec::new();

            while let Some(chunk) = stream.next().await {
                match chunk? {
                    StreamingChoice::Message(text) => {
                        match self.stdout {
                            true => {
                                print!("{}", text);
                                std::io::stdout().flush()?;
                            }
                            false => {
                                if let Some(tx) = &tx {
                                    tx.send(LoopResponse::Message(
                                        text.clone(),
                                    ))
                                    .await?;
                                }
                            }
                        }
                        current_response.push_str(&text);
                    }
                    StreamingChoice::ToolCall(name, _id, params) => {
                        if self.stdout {
                            println!(
                                "\nCalling tool: {} with params: {}",
                                name, params
                            );
                        }

                        let result = self
                            .agent
                            .tools
                            .call(&name, params.to_string())
                            .await?;

                        if self.stdout {
                            println!("Tool result: {}", result);
                        }

                        tool_results.push((name, result.to_string()));
                    }
                }
            }

            // Add assistant's response to message history
            if !current_response.is_empty() {
                current_messages.push(Message {
                    role: "assistant".to_string(),
                    content: current_response,
                });
            }

            // Add tool results to message history and send them
            if !tool_results.is_empty() {
                for (tool_name, result) in tool_results {
                    current_messages.push(Message {
                        role: "user".to_string(),
                        content: format!(
                            "Tool {} result: {}",
                            tool_name, result
                        ),
                    });

                    if let Some(tx) = &tx {
                        tx.send(LoopResponse::ToolCall {
                            name: tool_name,
                            result,
                        })
                        .await?;
                    }
                }
            } else {
                // No more tool calls, we can exit the loop
                break;
            }
        }

        Ok(current_messages)
    }

    pub fn with_stdout(mut self, enabled: bool) -> Self {
        self.stdout = enabled;
        self
    }
}
