use anyhow::Result;
use listen_kit::agents::listen::create_deep_research_agent_claude;
use listen_kit::agents::listen::create_deep_research_agent_gemini;
use listen_kit::common::spawn_with_signer;
use listen_kit::evm::util::env;
use listen_kit::reasoning_loop::Model;
use listen_kit::reasoning_loop::ReasoningLoop;
use listen_kit::signer::solana::LocalSolanaSigner;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let leader_reasoning_loop = match env("MODEL").as_str() {
        "gemini" => ReasoningLoop::new(Model::Gemini(Arc::new(
            create_deep_research_agent_gemini("en".to_string()),
        ))),
        "claude" => ReasoningLoop::new(Model::Claude(Arc::new(
            create_deep_research_agent_claude("en".to_string()),
        ))),
        _ => anyhow::bail!("Invalid model"),
    }
    .with_stdout(false);

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            println!("{:?}", response);
        }
    });

    let result = spawn_with_signer(Arc::new(signer), || async move {
        leader_reasoning_loop
            .stream(
                "what can you tell me about listen?".to_string(),
                vec![],
                Some(tx),
                None,
            )
            .await
    })
    .await;

    let _ = result.await?;

    Ok(())
}
