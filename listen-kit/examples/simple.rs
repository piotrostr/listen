#[cfg(feature = "solana")]
use {
    anyhow::Result,
    listen_kit::solana::tools::{initialize, Portfolio},
    listen_kit::solana::util::env,
    rig::streaming::{stream_to_stdout, StreamingPrompt},
};

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    initialize(env("SOLANA_PRIVATE_KEY")).await;

    let agent = rig::providers::anthropic::Client::from_env()
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
        .preamble("you are a portfolio checker, if you do wanna call a tool, outline the reasoning why that tool")
        .max_tokens(1024)
        .tool(Portfolio)
        .build();

    let mut stream = agent
        .stream_prompt("whats the portfolio looking like?")
        .await?;

    stream_to_stdout(agent, &mut stream).await?;

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    println!("enable the solana feature to run this example");
}
