use anyhow::Result;
use listen_kit::agents::listen::create_listen_agent_deepseek;
use listen_kit::common::spawn_with_signer;
use listen_kit::evm::util::env;
use listen_kit::reasoning_loop::Model;
use listen_kit::reasoning_loop::ReasoningLoop;
use listen_kit::signer::solana::LocalSolanaSigner;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let reasoning_loop = ReasoningLoop::new(Model::DeepSeek(Arc::new(
        create_listen_agent_deepseek("en".to_string()),
    )))
    .with_stdout(false);

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            print!("{:?}\n", response);
        }
    });

    let result = spawn_with_signer(Arc::new(signer), || async move {
        reasoning_loop
            .stream(
                "what can you tell me about listen?".to_string(),
                vec![],
                Some(tx),
            )
            .await
    })
    .await;

    let _ = result.await?;

    Ok(())
}
