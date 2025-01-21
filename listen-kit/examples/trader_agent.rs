use std::io::Write;

use anyhow::Result;
use listen_kit::agent::create_trader_agent;
use rig::agent::Agent;
use rig::completion::{Chat, Message};
use rig::providers::anthropic::completion::CompletionModel;

struct AgentWrapper {
    agent: Agent<CompletionModel>,
}

impl AgentWrapper {
    fn new(agent: Agent<CompletionModel>) -> Self {
        Self { agent }
    }

    async fn chat(
        &self,
        message: &str,
        chat_history: Vec<Message>,
    ) -> Result<String> {
        let response = self.agent.chat(message, chat_history.clone()).await?;

        // Always feed tool outputs back to the model
        let follow_up_prompt =
            format!("The output of the function call: {}", response);

        let mut updated_history = chat_history;
        updated_history.push(Message {
            role: "user".to_string(),
            content: message.to_string(),
        });
        updated_history.push(Message {
            role: "assistant".to_string(),
            content: response,
        });

        let interpreted_response =
            self.agent.chat(&follow_up_prompt, updated_history).await?;
        Ok(interpreted_response)
    }
    async fn chat_loop(&self) -> Result<()> {
        let mut chat_history = Vec::new();

        loop {
            print!("> ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("exit")
                || input.eq_ignore_ascii_case("quit")
            {
                println!("Goodbye!");
                break;
            }

            match self.chat(input, chat_history.clone()).await {
                Ok(response) => {
                    println!("🤖 {}", response);

                    // Update chat history
                    chat_history.push(Message {
                        role: "user".to_string(),
                        content: input.to_string(),
                    });
                    chat_history.push(Message {
                        role: "assistant".to_string(),
                        content: response,
                    });
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trader_agent = create_trader_agent().await?;
    let wrapped_agent = AgentWrapper::new(trader_agent);

    wrapped_agent.chat_loop().await?;

    Ok(())
}
