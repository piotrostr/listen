#[cfg(feature = "solana")]
use {
    anyhow::Result, listen_kit::agent::create_solana_agent,
    listen_kit::reasoning_loop::ReasoningLoop,
    listen_kit::signer::solana::LocalSolanaSigner,
    listen_kit::signer::SignerContext, listen_kit::solana::util::env,
    rig::completion::Message, std::sync::Arc,
};

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let trader_agent = create_solana_agent().await?;
        let wrapped_agent = ReasoningLoop::new(trader_agent);

        wrapped_agent
            .run(vec![Message {
                role: "user".to_string(),
                content: "what is the liquidity in the pool for my largest holding?"
                    .to_string(),
            }])
            .await?;

        Ok(())
    })
    .await?;

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    println!("enable the 'solana' feature to run this example.");
}
