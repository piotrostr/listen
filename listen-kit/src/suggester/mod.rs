use crate::common::gemini_agent_builder;
use anyhow::Result;
use rig::{
    completion::Prompt,
    message::{AssistantContent, Message, ToolResultContent, UserContent},
};

const PROMPT_EN: &str = r#"
Based on this conversation, predict 2-3 most likely concrete user responses.
Focus on direct answers, not questions.
One per line, keep each response short and specific.
For questions with options, predict the most likely option the user would choose.
Provide only the predictions, no other text.
"#;

const PROMPT_ZH: &str = r#"
根据此对话，预测2-3个最可能的具体用户回应。
注重直接的答复，而不是问题。
每行一个回应，保持简短具体。
对于有选项的问题，预测用户最可能选择的选项。
仅提供预测内容，不要其他文字。
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

    // TODO fix empty assistant message in the reasoning loop
    #[tokio::test]
    async fn test_suggest_real_case() {
        let messages = vec![
            Message::user("my balnace".to_string()),
            Message::assistant(
                "Your SOL balance is 0.19 SOL. That's like having a few bucks in your digital wallet! Anything I can help you with today?\n"
                    .to_string(),
            ),
            Message::user("Explore DeFi earning options".to_string()),
            Message::assistant(
                "I can't directly explore DeFi earning options for you in the sense of connecting to different platforms and displaying yields. However, I can help you find potential tokens to invest in that might be related to DeFi, or help you analyze existing tokens in your portfolio.\n\nTo give you the best suggestions, could you tell me:\n\n1.  **What kind of risk are you comfortable with?** (High, Medium, Low)\n2.  **Are there any specific DeFi sectors you're interested in?** (e.g., Lending, DEXs, Yield Aggregators)\n3.  **What's your timeframe?** (Are you looking for short-term opportunities or something more long-term?)\n\nIn the meantime, I can analyze tokens already in your portfolio, if you would like.\n"
                    .to_string(),
            ),
        ];

        let suggestions = suggest(&messages, "en").await.unwrap();
        println!("{:?}", suggestions);
    }
}
