#[cfg(feature = "solana")]
use {
    anyhow::Result, listen_kit::reasoning_loop::ReasoningLoop,
    listen_kit::signer::solana::LocalSolanaSigner,
    listen_kit::signer::SignerContext,
    listen_kit::solana::agent::create_solana_agent,
    listen_kit::solana::util::env, std::sync::Arc,
};

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    use listen_kit::{reasoning_loop::Model, solana::agent::Features};

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let trader_agent = Arc::new(
            create_solana_agent(None, Features { autonomous: false }).await?,
        );
        println!(
            "total tools available: {}",
            trader_agent.tools.schemas().unwrap().len()
        );
        let trader_agent = ReasoningLoop::new(Model::Anthropic(trader_agent))
            .with_stdout(true);

        trader_agent
            .stream(
                "
                we are testing the reasoning loop, constantly check my usdc
                balance and get quote for 0.01 sol into usdc
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
    println!("enable the 'solana' feature to run this example.");
}
