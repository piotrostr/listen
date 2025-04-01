#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use {
        listen_kit::reasoning_loop::ReasoningLoop,
        listen_kit::signer::solana::LocalSolanaSigner,
        listen_kit::signer::SignerContext, listen_kit::solana::util::env,
        std::sync::Arc,
    };

    use listen_kit::{
        reasoning_loop::Model,
        solana::agent::{create_solana_agent_gemini, Features},
    };

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let trader_agent = ReasoningLoop::new(Model::Gemini(Arc::new(
            create_solana_agent_gemini(
                None,
                Features {
                    autonomous: false,
                    deep_research: false,
                },
            ),
        )))
        .with_stdout(true);

        trader_agent
            .stream(
                "
                can you check the chart of Fartcoin please (search for the mint)
                "
                .to_string(),
                vec![],
                None,
            )
            .await?;

        Ok(())
    })
    .await?;

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    tracing::warn!("enable the 'solana' feature to run this example.");
}
