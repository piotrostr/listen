use crate::common::gemini_agent_builder;
use anyhow::Result;
use rig::completion::Prompt;

pub async fn extract_key_information(output: String) -> Result<String> {
    let agent = gemini_agent_builder().preamble("Extract the key information from the given output. Keep your answer brief").build();
    let res = agent.prompt(output.clone()).await.map_err(|e| {
        anyhow::anyhow!("Error extracting key information: {}", e)
    })?;

    tracing::info!(
        "[KEY INFORMATION] extract key information input: {}",
        output
    );

    tracing::info!(
        "[KEY INFORMATION] extract key information result: {}",
        res
    );

    Ok(res)
}
