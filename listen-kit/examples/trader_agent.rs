use anyhow::Result;
use listen_kit::agent::create_trader_agent;
use rig::cli_chatbot::cli_chatbot;

#[tokio::main]
async fn main() -> Result<()> {
    let trader_agent = create_trader_agent().await?;

    cli_chatbot(trader_agent).await?;

    Ok(())
}
