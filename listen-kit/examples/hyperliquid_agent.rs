#[cfg(feature = "hype")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use {
        listen_kit::reasoning_loop::ReasoningLoop,
        listen_kit::signer::evm::LocalEvmSigner,
        listen_kit::signer::SignerContext, std::sync::Arc,
    };

    use listen_kit::{
        agent::Features, evm::util::env, hype::create_hype_agent_openrouter,
        reasoning_loop::Model,
    };

    let prompt = "if i were to buy 1000 eth, what would the average price of my order be?"
        .to_string();

    let signer = LocalEvmSigner::new(env("ETHEREUM_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let model =
            Model::OpenRouter(Arc::new(create_hype_agent_openrouter(
                None,
                Features::default(),
                "en".to_string(),
            )));

        let agent = ReasoningLoop::new(model).with_stdout(true);

        let messages = agent
            .stream(
                prompt,
                vec![],
                None,
                None,
                "replace-with-any-persistant-user-id".to_string(),
            )
            .await?;

        tracing::info!(
            "messages: {}",
            serde_json::to_string_pretty(&messages).unwrap()
        );

        Ok(())
    })
    .await?;

    Ok(())
}

#[cfg(not(feature = "hype"))]
fn main() {
    tracing::warn!("enable the 'hype' feature to run this example.");
}
