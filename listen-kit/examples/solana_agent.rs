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
        solana::agent::{
            create_solana_agent_claude, create_solana_agent_deepseek,
            create_solana_agent_gemini, create_solana_agent_openrouter,
            Features,
        },
    };

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let features = Features {
            autonomous: false,
            deep_research: false,
        };
        let model = match std::env::var("MODEL").unwrap_or_default().as_str()
        {
            "gemini" => Model::Gemini(Arc::new(create_solana_agent_gemini(
                None,
                features,
                "en".to_string(),
            ))),
            "openrouter" => {
                Model::OpenRouter(Arc::new(create_solana_agent_openrouter(
                    None,
                    features,
                    "en".to_string(),
                    Some("deepseek/deepseek-chat-v3-0324".to_string()),
                )))
            }
            "deepseek" => {
                Model::DeepSeek(Arc::new(create_solana_agent_deepseek(
                    None,
                    features,
                    "en".to_string(),
                )))
            }
            "claude" => Model::Claude(Arc::new(create_solana_agent_claude(
                None,
                features,
                "en".to_string(),
            ))),
            _ => Model::Gemini(Arc::new(create_solana_agent_gemini(
                None,
                features,
                "en".to_string(),
            ))),
        };

        let trader_agent = ReasoningLoop::new(model).with_stdout(true);

        let _messages = trader_agent
            .stream(
                // "we are testing the resoning loop, fetch my solana balance, then fetch my the current time, repeat three times, batches of double tool calls please"
                "we are testing parallel tool calls, please check the current time 5 times, then my solana balance, also 5 times"
                .to_string(),
                vec![],
                None,
            )
            .await?;

        // tracing::info!(
        //     "messages: {}",
        //     serde_json::to_string_pretty(&messages).unwrap()
        // );

        Ok(())
    })
    .await?;

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    tracing::warn!("enable the 'solana' feature to run this example.");
}
