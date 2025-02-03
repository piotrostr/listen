use anyhow::Result;
use futures_util::StreamExt;
use rig::agent::Agent;
use rig::completion::Message;
use rig::providers::anthropic::completion::CompletionModel;
use rig::streaming::{StreamingChat, StreamingChoice};
use std::io::Write;

pub struct ReasoningLoop {
    agent: Agent<CompletionModel>,
    stdout: bool,
}

impl ReasoningLoop {
    pub fn new(agent: Agent<CompletionModel>) -> Self {
        Self {
            agent,
            stdout: true, // Default to true, could make this configurable
        }
    }

    pub async fn run(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
        let mut current_messages = messages;

        loop {
            let mut stream =
                self.agent.stream_chat("", current_messages.clone()).await?;

            let mut current_response = String::new();
            let mut tool_results = Vec::new();

            while let Some(chunk) = stream.next().await {
                match chunk? {
                    StreamingChoice::Message(text) => {
                        if self.stdout {
                            print!("{}", text);
                            std::io::stdout().flush()?;
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

            // Add tool results to message history if any
            if !tool_results.is_empty() {
                for (tool_name, result) in tool_results {
                    current_messages.push(Message {
                        role: "user".to_string(),
                        content: format!(
                            "Tool {} result: {}",
                            tool_name, result
                        ),
                    });
                }
            } else {
                // Print newline after completion if stdout is enabled
                if self.stdout {
                    println!();
                }
                // No more tool calls, we can exit the loop
                break;
            }
        }

        Ok(current_messages)
    }

    // Optional: Add method to configure stdout printing
    pub fn with_stdout(mut self, enabled: bool) -> Self {
        self.stdout = enabled;
        self
    }
}
