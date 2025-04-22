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
        agent::{model_to_versioned_model, Features},
        reasoning_loop::Model,
        solana::agent::{
            create_solana_agent_claude, create_solana_agent_deepseek,
            create_solana_agent_gemini, create_solana_agent_openai,
            create_solana_agent_openrouter,
        },
    };

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let features = Features {
            autonomous: false,
            deep_research: false,
            memory: false,
        };
        let model = match std::env::var("MODEL").unwrap_or_default().as_str()
        {
            "gemini" => Model::Gemini(Arc::new(create_solana_agent_gemini(
                None,
                features,
                "en".to_string(),
            ))),
            "openrouter-llama" => {
                Model::OpenRouter(Arc::new(create_solana_agent_openrouter(
                    None,
                    features,
                    "en".to_string(),
                    Some(model_to_versioned_model("llama".to_string())),
                )))
            }
            "openrouter-claude" => {
                Model::OpenRouter(Arc::new(create_solana_agent_openrouter(
                    None,
                    features,
                    "en".to_string(),
                    Some(model_to_versioned_model("claude".to_string())),
                )))
            }
            "openrouter-gemini" => {
                Model::OpenRouter(Arc::new(create_solana_agent_openrouter(
                    None,
                    features,
                    "en".to_string(),
                    Some(model_to_versioned_model("gemini".to_string())),
                )))
            }
            "openrouter-openai" => {
                Model::OpenRouter(Arc::new(create_solana_agent_openrouter(
                    None,
                    features,
                    "en".to_string(),
                    Some(model_to_versioned_model("openai".to_string())),
                )))
            }
            "openrouter-deepseek" => {
                Model::OpenRouter(Arc::new(create_solana_agent_openrouter(
                    None,
                    features,
                    "en".to_string(),
                    Some(model_to_versioned_model("deepseek".to_string())),
                )))
            }
            "openai" => {
                Model::OpenAI(Arc::new(create_solana_agent_openai(
                    None,
                    features,
                    "en".to_string(),
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

        let messages = trader_agent
            .stream(
                // "we are testing the resoning loop, fetch my solana balance, then fetch my the current time, repeat three times, batches of double tool calls please"
                "we are testing parallel tool calls, check my solana balance and USDC balance (EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v), do this in one response please"
                .to_string(),
                vec![],
                None,
                None,
                "replace-with-any-persistant-user-id".to_string(),
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
