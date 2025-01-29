#[cfg(feature = "solana")]
use {
    anyhow::{anyhow, Result},
    listen_kit::agent::{create_data_agent, create_trader_agent},
    listen_kit::common::wrap_unsafe,
    rig::agent::Agent,
    rig::cli_chatbot::cli_chatbot,
    rig::completion::Prompt,
    rig::providers::anthropic::completion::CompletionModel,
    rig_tool_macro::tool,
    std::sync::Arc,
    tokio::sync::OnceCell,
};

#[cfg(feature = "solana")]
static DATA_AGENT: OnceCell<Arc<Agent<CompletionModel>>> =
    OnceCell::const_new();
#[cfg(feature = "solana")]
static TRADER_AGENT: OnceCell<Arc<Agent<CompletionModel>>> =
    OnceCell::const_new();

#[cfg(feature = "solana")]
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

#[cfg(feature = "solana")]
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

#[cfg(feature = "solana")]
#[tool]
async fn data_action(prompt: String) -> Result<String> {
    let agent = get_data_agent().await;
    wrap_unsafe(move || async move {
        agent.prompt(&prompt).await.map_err(|e| anyhow!("{:#?}", e))
    })
    .await
}

#[cfg(feature = "solana")]
#[tool]
async fn trade_action(prompt: String) -> Result<String> {
    let agent = get_trader_agent().await;
    wrap_unsafe(move || async move {
        agent.prompt(&prompt).await.map_err(|e| anyhow!("{:#?}", e))
    })
    .await
}

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    use listen_kit::signer::solana::LocalSolanaSigner;
    use listen_kit::signer::SignerContext;
    use listen_kit::solana::util::env;

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));
    SignerContext::with_signer(Arc::new(signer), async {
        let leader = rig::providers::openai::Client::from_env()
            .agent(rig::providers::openai::GPT_4O)
            .preamble(
                "you are a swarm leader, you have a data agent to redirect 
                all of the user prompts that require looking for data and trader 
                agent to perform any trading actions, use your swarm accordingly",
            )
            .tool(DataAction)
            .tool(TradeAction)
            .build();

        cli_chatbot(leader).await?;
        Ok(())
    })
    .await?;

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    println!("enable the 'solana' feature to run this example");
}
