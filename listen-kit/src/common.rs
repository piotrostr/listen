use crate::reasoning_loop::{ReasoningLoop, StreamResponse};
use anyhow::{anyhow, Result};
use rig::message::{
    AssistantContent, Message, ToolResultContent, UserContent,
};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

use crate::signer::{SignerContext, TransactionSigner};
pub async fn wrap_unsafe<F, Fut, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let (tx, mut rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let result = f().await;
        let _ = tx.send(result).await;
    });

    rx.recv().await.ok_or_else(|| anyhow!("Channel closed"))?
}

pub async fn spawn_with_signer<F, Fut, T>(
    signer: Arc<dyn TransactionSigner>,
    f: F,
) -> tokio::task::JoinHandle<Result<T>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        SignerContext::with_signer(signer, async { f().await }).await
    })
}

use rig::agent::{Agent, AgentBuilder};
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;
use rig::providers::gemini::completion::CompletionModel as GeminiCompletionModel;

pub type GeminiAgent = rig::agent::Agent<GeminiCompletionModel>;
pub type ClaudeAgent = rig::agent::Agent<AnthropicCompletionModel>;

pub fn claude_agent_builder() -> AgentBuilder<AnthropicCompletionModel> {
    rig::providers::anthropic::Client::from_env()
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
        .max_tokens(1024 * 4)
}

pub async fn plain_agent() -> Result<Agent<AnthropicCompletionModel>> {
    Ok(claude_agent_builder()
        .preamble("be nice to the users")
        .max_tokens(1024 * 4)
        .build())
}

pub fn gemini_agent_builder() -> AgentBuilder<GeminiCompletionModel> {
    rig::providers::gemini::Client::from_env()
        .agent(rig::providers::gemini::completion::GEMINI_2_0_FLASH)
        .max_tokens(1024 * 4)
}

pub const PREAMBLE_COMMON: &str = "";

pub fn messages_to_string(messages: &[Message], max_chars: usize) -> String {
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

pub fn role(message: &Message) -> String {
    match message {
        Message::User { .. } => "user".to_string(),
        Message::Assistant { .. } => "assistant".to_string(),
    }
}

pub fn content(message: &Message) -> String {
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

pub async fn spawn_with_signer_and_channel<F, Fut, T>(
    signer: Arc<dyn TransactionSigner>,
    channel: Option<Sender<StreamResponse>>,
    f: F,
) -> tokio::task::JoinHandle<Result<T>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        SignerContext::with_signer(signer, async {
            ReasoningLoop::with_stream_channel(channel, f).await
        })
        .await
    })
}
