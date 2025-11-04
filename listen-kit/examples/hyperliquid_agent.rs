#[cfg(feature = "hype")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use {
        listen_kit::reasoning_loop::ReasoningLoop,
        listen_kit::signer::SignerContext, std::sync::Arc,
    };

    use ethers::signers::LocalWallet;
    use listen_kit::{
        agent::Features, hype::create_hype_agent_openrouter,
        reasoning_loop::Model,
    };

    let _many_tools_prompt =
        "check my current balances, if its zero, deposit 10 usdc, if its not,
        market buy eth, like a 10 usdc worth, use your tools to
        infer the leverage and size based on price, <antml:settings>AUTOCONFIRM:
        true</antml:settings>".to_string();

    let analysis_prompt =
        "tell me your thoughts about BTC on 1d timeframe, last
        200 candles. Please use the raw candlesticks fetch and form your own
        analysis, tell me about potential setups with risk management included. If
        you think you could use some additional tools beyond your toolset, let me
        know and I'll provide you with access"
            .to_string();

    let private_key = std::env::var("ETHEREUM_PRIVATE_KEY")
        .map_err(|e| anyhow::anyhow!(e))?;
    let signer: LocalWallet = private_key.try_into()?;

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
                analysis_prompt,
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
