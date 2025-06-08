#[cfg(feature = "hype")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use {
        listen_kit::reasoning_loop::ReasoningLoop,
        listen_kit::signer::SignerContext, std::sync::Arc,
    };

    use listen_kit::{
        agent::Features, hype::create_hype_agent_openrouter,
        reasoning_loop::Model, signer::privy::PrivySigner,
    };
    use privy::{auth::UserSession, config::PrivyConfig, Privy};

    let prompt =
        "check my current balances, if its zero, deposit 3 usdc".to_string();

    // TODO allow local signer (hyperliquid-rust-sdk uses ethers LocalWallet not alloy)
    let session = UserSession {
        user_id: "".to_string(),
        evm_wallet_id: Some("k0pq0k5an1fvo35m5gm3wn8d".to_string()),
        wallet_address: Some(
            "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770".to_string(),
        ),
        ..Default::default()
    };
    let signer = PrivySigner::new(
        Arc::new(Privy::new(PrivyConfig::from_env()?)),
        session,
        "en".to_string(),
    );

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
