use crate::common::gemini_agent_builder;
use anyhow::Result;
use rig::{
    completion::Prompt,
    message::{AssistantContent, Message, ToolResultContent, UserContent},
};

const PROMPT_EN: &str = r#"
Strictly based on this conversation, generate 2-3 follow-up questions for what I can do next. One per line:
"#;

const PROMPT_ZH: &str = r#"
严格基于这段对话，生成2-3个后续问题，告诉我接下来可以做什么。每行一个：
"#;

const MAX_CHARS: usize = 30000;

pub async fn suggest(
    messages: &[Message],
    locale: &str,
) -> Result<Vec<String>> {
    let prompt = if locale == "zh" {
        format!(
            "{}\n\n{}",
            PROMPT_ZH,
            messages_to_string(messages, MAX_CHARS)
        )
    } else {
        format!(
            "{}\n\n{}",
            PROMPT_EN,
            messages_to_string(messages, MAX_CHARS)
        )
    };

    let agent = gemini_agent_builder().build();

    let response = agent.prompt(Message::user(prompt)).await?;

    // Split response into lines and take exactly 4 suggestions
    let suggestions: Vec<String> = response
        .lines()
        .filter(|line| !line.is_empty())
        .take(3)
        .map(|s| s.trim().to_string())
        .collect();

    Ok(suggestions)
}

fn messages_to_string(messages: &[Message], max_chars: usize) -> String {
    let snippet = messages
        .iter()
        .map(|m| format!("{}: {}", role(m), content(m)))
        .collect::<Vec<_>>()
        .join("\n");

    if snippet.len() > max_chars {
        "...".to_string() + &snippet[snippet.len() - max_chars..]
    } else {
        snippet
    }
}

fn role(message: &Message) -> String {
    match message {
        Message::User { .. } => "user".to_string(),
        Message::Assistant { .. } => "assistant".to_string(),
    }
}

fn content(message: &Message) -> String {
    match message {
        Message::User { content } => match content.first() {
            UserContent::Text(text) => text.text.clone(),
            UserContent::ToolResult(tool_result) => {
                match tool_result.content.first() {
                    ToolResultContent::Text(text) => text.text.clone(),
                    _ => "".to_string(),
                }
            }
            UserContent::Image(_) => "".to_string(),
            UserContent::Audio(_) => "".to_string(),
            UserContent::Document(_) => "".to_string(),
        },
        Message::Assistant { content } => match content.first() {
            AssistantContent::Text(text) => text.text.clone(),
            AssistantContent::ToolCall(tool_call) => {
                let call = format!(
                    "called {} with {}",
                    tool_call.function.name, tool_call.function.arguments
                );
                call
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rig::message::Message;

    #[tokio::test]
    async fn test_suggest_en() {
        let messages = vec![
            Message::user("How does Solana achieve high transaction throughput?".to_string()),
            Message::assistant(
                "Solana achieves high throughput through parallel processing and Proof of History.".to_string(),
            ),
        ];

        let suggestions = suggest(&messages, "en").await.unwrap();
        println!("{:?}", suggestions);
    }

    #[tokio::test]
    async fn test_suggest_zh() {
        let messages = vec![
            Message::user("Solana如何实现高交易吞吐量？".to_string()),
            Message::assistant(
                "Solana通过并行处理和历史证明机制实现高吞吐量。".to_string(),
            ),
        ];

        let suggestions = suggest(&messages, "zh").await.unwrap();
        println!("{:?}", suggestions);
    }
}
