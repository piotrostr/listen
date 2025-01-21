use anyhow::Result;
use listen_kit::tools::{initialize, Portfolio};
use listen_kit::util::env;
use rig::completion::Prompt;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    initialize(env("PRIVATE_KEY")).await;

    let agent = rig::providers::openai::Client::from_env()
        .agent(rig::providers::openai::GPT_4_TURBO)
        .preamble("you are a portfolio checker")
        .max_tokens(1024)
        .tool(Portfolio)
        .build();

    println!(
        "{}",
        agent.prompt("whats the portfolio looking like").await?
    );

    Ok(())
}
