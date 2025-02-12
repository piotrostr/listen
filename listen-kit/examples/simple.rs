#[cfg(feature = "solana")]
use {
    anyhow::Result,
    listen_kit::signer::solana::LocalSolanaSigner,
    listen_kit::signer::SignerContext,
    listen_kit::solana::util::env,
    std::sync::Arc,
    listen_kit::reasoning_loop::ReasoningLoop,
    listen_kit::solana::tools::GetPortfolio,
    rig::{message::Message, OneOrMany, message::UserContent},
};

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));
    SignerContext::with_signer(Arc::new(signer), async {
        let agent = rig::providers::anthropic::Client::from_env()
            .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
            .preamble("you are a portfolio checker, if you do wanna call a tool, outline the reasoning why that tool")
            .max_tokens(1024)
            .tool(GetPortfolio)
            .build();

        let agent = ReasoningLoop::new(Arc::new(agent));

        agent
            .stream(vec![Message::User {
                content: OneOrMany::one(UserContent::text(
                    "whats the portfolio looking like?"
                        .to_string(),
                )),
            }], None)
            .await?; 

        Ok(())

    }).await?;

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    println!("enable the solana feature to run this example");
}
