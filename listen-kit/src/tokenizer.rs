#[cfg(feature = "tokenizer")]
use std::sync::Mutex;

#[cfg(feature = "tokenizer")]
use anyhow::Result;
use rig::message::{
    AssistantContent, Message, ToolResultContent, UserContent,
};
#[cfg(feature = "tokenizer")]
use tokenizers::tokenizer::{EncodeInput, Tokenizer};
#[cfg(feature = "tokenizer")]
use tokenizers::InputSequence;

#[cfg(feature = "tokenizer")]
lazy_static::lazy_static! {
    static ref TOKENIZER: Mutex<Tokenizer> = Mutex::new(get_tokenizer());
}

#[cfg(feature = "tokenizer")]
pub fn get_tokenizer() -> Tokenizer {
    let tokenizer_data = include_bytes!("../claude-v3-tokenizer.json");

    Tokenizer::from_bytes(tokenizer_data).unwrap()
}

// Fast character-based estimation (English text averages ~4 chars per token)
pub fn estimate_tokens(text: &str) -> usize {
    // Average ratio of characters to tokens for English text
    const CHARS_PER_TOKEN: f32 = 4.0;

    (text.chars().count() as f32 / CHARS_PER_TOKEN).ceil() as usize
}

// Estimate tokens for a conversation based on character count
pub fn estimate_conversation_tokens(
    prompt: &str,
    messages: &[Message],
) -> usize {
    let mut estimated_tokens = estimate_tokens(prompt);

    for message in messages {
        match message {
            Message::User { content } => match content.first() {
                UserContent::Text(text) => {
                    estimated_tokens += estimate_tokens(&text.text);
                }
                UserContent::ToolResult(tool_result) => {
                    if let ToolResultContent::Text(text) =
                        tool_result.content.first()
                    {
                        estimated_tokens += estimate_tokens(&text.text);
                    }
                }
                _ => {}
            },
            Message::Assistant { content } => {
                if let AssistantContent::Text(text) = content.first() {
                    estimated_tokens += estimate_tokens(&text.text);
                }
            }
        }
    }

    estimated_tokens
}

// Check if a conversation exceeds token limit using character-based estimation
pub fn exceeds_token_limit(
    prompt: &str,
    messages: &[Message],
    limit: usize,
) -> bool {
    // Use a conservative ratio to avoid false negatives
    const SAFETY_FACTOR: f32 = 0.8;

    let estimated_tokens = estimate_conversation_tokens(prompt, messages);
    let adjusted_limit = (limit as f32 * SAFETY_FACTOR) as usize;

    let result = estimated_tokens > adjusted_limit;

    tracing::info!("Context estimated at {} tokens", estimated_tokens);

    if result {
        tracing::warn!(
            "Context estimated at {} tokens > {} limit",
            estimated_tokens,
            adjusted_limit
        );
    }

    result
}

// Keep these for compatibility, but they won't be used in the fast path
#[cfg(feature = "tokenizer")]
pub fn tokenize(text: &str) -> Result<Vec<(u32, String)>> {
    let tokenizer = TOKENIZER.lock().unwrap();

    let val = EncodeInput::Single(InputSequence::Raw(text.into()));

    let encoded_text = tokenizer.encode(val, false);

    match encoded_text {
        Ok(encoded_text) => Ok(encoded_text
            .get_ids()
            .iter()
            .zip(encoded_text.get_tokens().iter().cloned())
            .map(|(id, token)| (*id, token.to_string()))
            .collect()),
        Err(err) => Err(anyhow::Error::msg(err.to_string())),
    }
}

#[cfg(feature = "tokenizer")]
pub fn count_tokens(text: &str) -> Result<usize> {
    let tokenizer = TOKENIZER.lock().unwrap();

    let val = EncodeInput::Single(InputSequence::Raw(text.into()));

    let encoded_text = tokenizer.encode(val, false);

    match encoded_text {
        Ok(encoded_text) => Ok(encoded_text.len()),
        Err(err) => Err(anyhow::Error::msg(err.to_string())),
    }
}
