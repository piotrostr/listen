#[cfg(feature = "solana")]
use {
    anyhow::Result,
    futures::StreamExt,
    listen_kit::agent::create_trader_agent,
    listen_kit::signer::solana::LocalSolanaSigner,
    listen_kit::signer::SignerContext,
    listen_kit::solana::util::env,
    rig::agent::Agent,
    rig::completion::Message,
    rig::providers::anthropic::completion::CompletionModel,
    rig::streaming::{StreamingChat, StreamingChoice},
    std::io::Write,
    std::sync::Arc,
};

#[cfg(feature = "solana")]
const MAX_RETRIES: usize = 3;
#[cfg(feature = "solana")]
const CONTINUE_PROMPT: &str = "
Based on the previous tool results:
1. Do you need any additional information?
2. If yes, what tool calls are needed and why?
3. If no, provide your final response.
this is just a preamble, user won't see this message, it is for your reasoning
";

#[cfg(feature = "solana")]
#[derive(Default)]
pub struct ReasoningState {
    tool_calls: Vec<String>,
    intermediate_results: Vec<String>,
    final_response: Option<String>,
}

#[cfg(feature = "solana")]
#[derive(Default)]
struct FormattedResponse {
    final_answer: String,
    tool_calls: Vec<String>,
}

#[cfg(feature = "solana")]
struct AgentWrapper {
    pub agent: Agent<CompletionModel>,
}

#[cfg(feature = "solana")]
impl AgentWrapper {
    fn new(agent: Agent<CompletionModel>) -> Self {
        Self { agent }
    }

    async fn handle_tool_call(
        &self,
        name: &str,
        params: &str,
        retries: usize,
    ) -> Result<String> {
        if retries >= MAX_RETRIES {
            return Err(anyhow::anyhow!("Max retries reached for tool call"));
        }

        match self.agent.tools.call(name, params.to_string()).await {
            Ok(res) => Ok(res),
            Err(e) => {
                if retries < MAX_RETRIES {
                    Box::pin(self.handle_tool_call(name, params, retries + 1))
                        .await
                } else {
                    Err(e.into())
                }
            }
        }
    }

    fn format_response(&self, state: ReasoningState) -> FormattedResponse {
        FormattedResponse {
            final_answer: state.final_response.unwrap_or_default(),
            tool_calls: state.tool_calls,
        }
    }

    async fn stream_chat(
        &self,
        message: &str,
        chat_history: Vec<Message>,
    ) -> Result<FormattedResponse> {
        let mut current_prompt = message.to_string();
        let mut current_history = chat_history;
        let max_iterations = 15;
        let mut iteration = 0;
        let mut state = ReasoningState::default();

        while iteration < max_iterations {
            iteration += 1;

            let mut stream = self
                .agent
                .stream_chat(&current_prompt, current_history.clone())
                .await?;

            let mut last_was_tool_call = false;
            let mut current_segment = String::new();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(StreamingChoice::Message(text)) => {
                        print!("{}", text);
                        std::io::stdout().flush()?;
                        current_segment.push_str(&text);
                        last_was_tool_call = false;
                    }
                    Ok(StreamingChoice::ToolCall(name, _, params)) => {
                        state.tool_calls.push(name.clone());
                        let res = self
                            .handle_tool_call(&name, &params.to_string(), 0)
                            .await?;
                        println!("\nTool Result: {}", res);
                        current_segment.push_str(&format!(
                            "{}({}): {}",
                            &name,
                            &params.to_string(),
                            res
                        ));
                        state.intermediate_results.push(res);
                        last_was_tool_call = true;
                    }
                    Err(e) => {
                        eprintln!("\nError: {}", e);
                        return Err(e.into());
                    }
                }
            }

            if !last_was_tool_call {
                state.final_response = Some(current_segment.clone());
                break;
            }

            // Update history with the latest interaction and continue
            current_history.push(Message {
                role: "user".to_string(),
                content: current_prompt.clone(),
            });
            current_history.push(Message {
                role: "assistant".to_string(),
                content: current_segment.clone(),
            });

            // Set new prompt for the next iteration
            current_prompt = CONTINUE_PROMPT.to_string();
        }

        if iteration >= max_iterations {
            println!("\nReached maximum number of iterations.");
        }

        println!();
        Ok(self.format_response(state))
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

            match self.stream_chat(input, chat_history.clone()).await {
                Ok(response) => {
                    // Update chat history
                    chat_history.push(Message {
                        role: "user".to_string(),
                        content: input.to_string(),
                    });
                    chat_history.push(Message {
                        role: "assistant".to_string(),
                        content: format!(
                            "{} ({})",
                            response.final_answer,
                            response.tool_calls.join(", ")
                        ),
                    });
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        Ok(())
    }
}

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));
    SignerContext::with_signer(Arc::new(signer), async {
        let trader_agent = create_trader_agent().await?;
        let wrapped_agent = AgentWrapper::new(trader_agent);

        wrapped_agent.chat_loop().await?;

        Ok(())
    })
    .await?;

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    println!("enable the 'solana' feature to run this example.");
}
