#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use listen_kit::{
        reasoning_loop::Model,
        solana::agent::{create_solana_agent_deepseek, Features},
    };
    use {
        listen_kit::reasoning_loop::ReasoningLoop,
        listen_kit::signer::solana::LocalSolanaSigner,
        listen_kit::signer::SignerContext, listen_kit::solana::util::env,
        std::sync::Arc,
    };

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    dotenv::dotenv().ok();

    SignerContext::with_signer(Arc::new(signer), async {
        let trader_agent = Arc::new(create_solana_agent_deepseek(
            None,
            Features {
                autonomous: false,
                deep_research: false,
            },
        ));
        for tool in trader_agent.tools.schemas().iter() {
            tracing::info!("{}", serde_json::to_string(tool).unwrap());
        }
        let trader_agent =
            ReasoningLoop::new(Model::DeepSeek(trader_agent)).with_stdout(true);

        let messages = trader_agent
            .stream(
                "
                we are testing the resoning loop, first grab my solana pubkey then my solana balance,
                then get metadata for Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump, the repeat the operation 3 times
                "
                .to_string(),
                vec![],
                None,
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

#[cfg(not(feature = "solana"))]
fn main() {
    tracing::warn!("enable the 'solana' feature to run this example.");
}
