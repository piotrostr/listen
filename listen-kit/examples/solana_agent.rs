#[cfg(feature = "solana")]
use {
    anyhow::Result, listen_kit::reasoning_loop::ReasoningLoop,
    listen_kit::signer::solana::LocalSolanaSigner,
    listen_kit::signer::SignerContext,
    listen_kit::solana::agent::create_solana_agent,
    listen_kit::solana::util::env, rig::completion::Message,
    rig::message::UserContent, rig::OneOrMany, std::sync::Arc,
};

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let trader_agent = Arc::new(create_solana_agent(None).await?);
        let trader_agent = ReasoningLoop::new(trader_agent).with_stdout(true);

        trader_agent
            .stream(vec![Message::User {
                content: OneOrMany::one(UserContent::text(
                    "what is the liquidity in the pool for my largest holding?"
                        .to_string(),
                )),
            }], None)
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
