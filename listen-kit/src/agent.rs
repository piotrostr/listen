use crate::{
    cross_chain::agent::create_cross_chain_agent,
    evm::agent::create_evm_agent,
    reasoning_loop::Model,
    solana::agent::{
        create_solana_agent_claude, create_solana_agent_deepseek,
        create_solana_agent_gemini, create_solana_agent_openai, Features,
    },
};
use std::sync::Arc;

pub fn create_agent(
    preamble: Option<String>,
    features: Features,
    locale: String,
    chain: String,
    model_type: String,
) -> Model {
    match chain.as_str() {
        "solana" => match model_type.as_str() {
            "claude" => Model::Claude(Arc::new(create_solana_agent_claude(
                preamble, features, locale,
            ))),
            "gemini" => Model::Gemini(Arc::new(create_solana_agent_gemini(
                preamble, features, locale,
            ))),
            "deepseek" => Model::DeepSeek(Arc::new(
                create_solana_agent_deepseek(preamble, features, locale),
            )),
            "openai" => Model::OpenAI(Arc::new(create_solana_agent_openai(
                preamble, features, locale,
            ))),
            _ => Model::Claude(Arc::new(create_solana_agent_claude(
                preamble, features, locale,
            ))),
        },
        "evm" => Model::Claude(Arc::new(create_evm_agent(preamble))),
        "omni" => Model::Claude(Arc::new(create_cross_chain_agent(preamble))),
        _ => Model::Claude(Arc::new(create_solana_agent_claude(
            preamble, features, locale,
        ))),
    }
}
