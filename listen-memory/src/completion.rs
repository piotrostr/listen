use anyhow::Result;
use regex::Regex;
use rig::completion::Prompt;

pub async fn generate_completion(prompt: &str) -> Result<String> {
    tracing::debug!(target: "listen-memory", "Generating completion for: {}", prompt);
    let model = rig::providers::gemini::Client::from_env()
        .agent(rig::providers::gemini::completion::GEMINI_2_0_FLASH)
        .build();

    let res = model.prompt(prompt).await.map_err(anyhow::Error::new)?;
    let parsed = extract_from_code_blocks_if_any(&res);
    tracing::debug!(target: "listen-memory", "Parsed: {}", parsed);
    Ok(parsed)
}

pub fn extract_from_code_blocks_if_any(content: &str) -> String {
    // Match everything between triple backticks, non-greedy
    let re = Regex::new(r"```(?:\w+)?\s*([\s\S]*?)\s*```").unwrap();
    let caps = re.captures(content);
    let result = caps.map_or_else(|| content.trim().to_string(), |c| c[1].trim().to_string());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_code_blocks_if_any() {
        let content = r#"```json
{
    "should_evolve": false,
    "actions": [],
    "suggested_connections": [],
    "tags_to_update": [],
    "new_context_neighborhood": [],
    "new_tags_neighborhood": []
}
```"#;
        let parsed = extract_from_code_blocks_if_any(content);
        let parse_result = serde_json::from_str::<serde_json::Value>(&parsed);
        assert!(
            parse_result.is_ok(),
            "Failed to parse JSON: {:?}",
            parse_result.err()
        );
    }
}
