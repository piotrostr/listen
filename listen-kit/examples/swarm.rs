use anyhow::{anyhow, Result};
use listen_kit::agent::{create_data_agent, create_trader_agent};
use listen_kit::util::wrap_unsafe;
use rig::agent::Agent;
use rig::cli_chatbot::cli_chatbot;
use rig::completion::Prompt;
use rig::providers::openai::CompletionModel;
use rig_tool_macro::tool;
use std::sync::Arc;
use tokio::sync::OnceCell;

static DATA_AGENT: OnceCell<Arc<Agent<CompletionModel>>> =
    OnceCell::const_new();
static TRADER_AGENT: OnceCell<Arc<Agent<CompletionModel>>> =
    OnceCell::const_new();

async fn get_data_agent() -> &'static Arc<Agent<CompletionModel>> {
    DATA_AGENT
        .get_or_init(|| async {
            Arc::new(
                create_data_agent()
                    .await
                    .expect("Failed to create data agent"),
            )
        })
        .await
}

async fn get_trader_agent() -> &'static Arc<Agent<CompletionModel>> {
    TRADER_AGENT
        .get_or_init(|| async {
            Arc::new(
                create_trader_agent()
                    .await
                    .expect("Failed to create trader agent"),
            )
        })
        .await
}

#[tool]
async fn data_action(prompt: String) -> Result<String> {
    let agent = get_data_agent().await;
    wrap_unsafe(move || async move {
        agent.prompt(&prompt).await.map_err(|e| anyhow!("{:#?}", e))
    })
    .await
}

#[tool]
async fn trade_action(prompt: String) -> Result<String> {
    let agent = get_trader_agent().await;
    wrap_unsafe(move || async move {
        agent.prompt(&prompt).await.map_err(|e| anyhow!("{:#?}", e))
    })
    .await
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let leader = rig::providers::openai::Client::from_env()
        .agent(rig::providers::openai::GPT_4O)
        .preamble("you are a swarm leader, you have a data agent to redirect all of the user prompts that require looking for data and trader agent to perform any trading actions, use your swarm accordingly")
        .tool(DataAction)
        .tool(TradeAction)
        .build();

    cli_chatbot(leader).await?;

    Ok(())
}
