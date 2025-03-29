use crate::tokenizer::exceeds_token_limit;
use anyhow::Result;
use rig::agent::Agent;
use rig::completion::Message;
use rig::providers::anthropic::completion::CompletionModel as AnthropicModel;
use rig::providers::gemini::completion::CompletionModel as GeminiModel;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::task_local;

pub mod anthropic;
pub mod gemini;
#[derive(Serialize, Debug, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum StreamResponse {
    Message(String),
    ToolCall {
        id: String,
        name: String,
        params: String,
    },
    ToolResult {
        id: String,
        name: String,
        result: String,
    },
    Error(String),
    NestedAgentOutput {
        agent_type: String,
        content: String,
    },
}

impl StreamResponse {
    pub fn stringify(&self) -> String {
        match self {
            StreamResponse::Message(message) => message.clone(),
            StreamResponse::ToolCall { id, name, params } => {
                format!("called {}({}) [ID: {}]", name, params, id)
            }
            StreamResponse::ToolResult { id, name, result } => {
                format!("{}({}) [ID: {}]", name, result, id)
            }
            StreamResponse::Error(error) => error.clone(),
            StreamResponse::NestedAgentOutput {
                agent_type,
                content,
            } => {
                format!("{}: {}", agent_type, content)
            }
        }
    }
}

#[derive(Clone)]
pub enum Model {
    Anthropic(Arc<Agent<AnthropicModel>>),
    Gemini(Arc<Agent<GeminiModel>>),
}

pub struct ReasoningLoop {
    model: Model,
    stdout: bool,
    tx: Option<Sender<StreamResponse>>,
}

impl ReasoningLoop {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            stdout: true,
            tx: None,
        }
    }

    pub async fn stream(
        &self,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
    ) -> Result<Vec<Message>> {
        if tx.is_none() && !self.stdout {
            panic!("enable stdout or provide tx channel");
        }

        // Simple character-based check for token limit
        if exceeds_token_limit(&prompt, &messages, 40_000) {
            return Err(anyhow::anyhow!(
                "Ahoy! Context is getting long, please start a new conversation",
            ));
        }

        // Use the task-local scope for the entire stream operation
        CURRENT_STREAM_CHANNEL
            .scope(RefCell::new(tx.clone()), async {
                match &self.model {
                    Model::Anthropic(agent) => {
                        self.stream_anthropic(agent, prompt, messages, tx)
                            .await
                    }
                    Model::Gemini(agent) => {
                        self.stream_gemini(agent, prompt, messages, tx).await
                    }
                }
            })
            .await
    }

    pub fn with_stdout(mut self, enabled: bool) -> Self {
        self.stdout = enabled;
        self
    }
}

// Define a task-local variable to hold the current stream channel
task_local! {
    static CURRENT_STREAM_CHANNEL: RefCell<Option<Sender<StreamResponse>>>;
}

// Function to get the current stream channel
pub async fn get_current_stream_channel() -> Option<Sender<StreamResponse>> {
    match CURRENT_STREAM_CHANNEL.try_with(|c| c.borrow().clone()) {
        Ok(channel) => channel,
        Err(_) => None,
    }
}

// Set the current stream channel
pub async fn set_current_stream_channel(
    channel: Option<Sender<StreamResponse>>,
) {
    let _ = CURRENT_STREAM_CHANNEL
        .scope(RefCell::new(channel), async {
            // Any code here will have access to the channel
        })
        .await;
}
