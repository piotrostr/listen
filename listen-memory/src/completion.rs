use anyhow::Result;
use rig::completion::Prompt;

pub async fn generate_completion(prompt: &str) -> Result<String> {
    let model = rig::providers::gemini::Client::from_env()
        .agent(rig::providers::gemini::completion::GEMINI_2_0_FLASH)
        .build();

    model.prompt(prompt).await.map_err(anyhow::Error::new)
}
