#[cfg(feature = "solana")]
use {
    anyhow::Result, listen_kit::reasoning_loop::ReasoningLoop,
    listen_kit::signer::solana::LocalSolanaSigner,
    listen_kit::signer::SignerContext,
    listen_kit::solana::agent::create_solana_agent_gemini,
    listen_kit::solana::util::env, std::sync::Arc,
};

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    use listen_kit::{reasoning_loop::Model, solana::agent::Features};

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    dotenv::dotenv().ok();

    SignerContext::with_signer(Arc::new(signer), async {
        let trader_agent = Arc::new(create_solana_agent_gemini(
            None,
            Features { autonomous: false },
        ));
        for tool in trader_agent.tools.schemas().iter() {
            println!("{}", serde_json::to_string(tool).unwrap());
        }
        let trader_agent =
            ReasoningLoop::new(Model::Gemini(trader_agent)).with_stdout(true);

        trader_agent
            .stream(
                "
                we are testing the resoning loop, first grab my solana pubkey then my solana balance,
                then search for Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump and research it on x
                don't give up when you encounter a roadblock, keep going strong
                you are autonomous and can call any number of tool to satisfy
                the requirements
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
