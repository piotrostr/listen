#[cfg(feature = "hype")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use {
        listen_kit::reasoning_loop::ReasoningLoop,
        listen_kit::signer::SignerContext, std::sync::Arc,
    };

    use ethers::signers::LocalWallet;
    use listen_kit::{
        agent::Features, hype::create_hype_agent_openrouter,
        reasoning_loop::Model,
    };

    let prompt =
        "check my current balances, if its zero, deposit 10 usdc, if its not,
        market buy eth, like a 10 usdc worth, use your tools to
        infer the leverage and size based on price, <antml:settings>AUTOCONFIRM:
        true</antml:settings>".to_string();

    let private_key = std::env::var("ETHEREUM_PRIVATE_KEY")
        .map_err(|e| anyhow::anyhow!(e))?;
    let signer: LocalWallet = private_key.try_into()?;

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
                "replace-with-any-persistent-user-id".to_string(),
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
