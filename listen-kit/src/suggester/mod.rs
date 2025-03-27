use crate::common::gemini_agent_builder;
use anyhow::Result;
use rig::{
    completion::Prompt,
    message::{AssistantContent, Message, ToolResultContent, UserContent},
};

const PREAMBLE_EN: &str = r#"You are a helpful AI assistant focused on
suggesting relevant follow-up questions.  Based on the conversation history,
dynamically suggest follow-up prompts that are most likely to be useful. Focus
on questions that explore underlying concepts, technical details, or practical
applications. Make suggestions concise and specific."#;

const PREAMBLE_ZH: &str = r#"你是一个专注于提供相关后续问题的AI助手。
根据对话历史，建议2个后续提示，这些提示有助于加深理解并触发额外研究。
重点关注探索基本概念、技术细节或实际应用的问题。建议要简洁具体。"#;

pub async fn suggest(
    messages: &[Message],
    locale: &str,
) -> Result<Vec<String>> {
    let preamble = if locale == "zh" {
        PREAMBLE_ZH
    } else {
        PREAMBLE_EN
    };

    let agent = gemini_agent_builder().preamble(preamble).build();
    let max_chars = 30000;

    let prompt = format!(
        "{}\n\nBased on this conversation, provide exactly 2 follow-up questions, one per line:",
        messages_to_string(messages, max_chars)
    );

    let response = agent.prompt(Message::user(prompt)).await?;

    // Split response into lines and take exactly 4 suggestions
    let suggestions: Vec<String> = response
        .lines()
        .filter(|line| !line.is_empty())
        .take(4)
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
