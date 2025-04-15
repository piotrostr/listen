use crate::common::ClaudeAgent;
use crate::common::DeepSeekAgent;
use crate::common::GeminiAgent;
use crate::common::OpenAIAgent;
use crate::common::OpenRouterAgent;
use crate::tokenizer::exceeds_token_limit;
use anyhow::Result;
use listen_memory::graph::GraphMemory;
use rig::completion::Message;
use rig::message::ToolCall;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::task_local;

pub mod debase64;
pub mod model;
pub mod stream_gemini;
pub mod stream_generic;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct SimpleToolResult {
    index: usize,
    id: String,
    name: String,
    params: String,
    result: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum StreamResponse {
    Message(String),
    ParToolCall {
        tool_calls: Vec<ToolCall>,
    },
    ParToolResult {
        tool_results: Vec<SimpleToolResult>,
    },
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
    pub fn render(&self) -> String {
        match self {
            StreamResponse::Message(message) => message.clone(),
            StreamResponse::ToolCall { name, params, .. } => {
                let params =
                    serde_json::from_str::<serde_json::Value>(params)
                        .unwrap_or_default();
                let params_str = match params {
                    serde_json::Value::Object(obj) => obj
                        .iter()
                        .map(|(k, v)| format!("- {}: {}", k, v))
                        .collect::<Vec<String>>()
                        .join("\n"),
                    _ => params.to_string(),
                };
                format!("\nCalling {} with:\n{}", name, params_str)
            }
            StreamResponse::ToolResult { result, .. } => {
                format!("\n\n{}", result)
            }
            StreamResponse::Error(error) => error.clone(),
            // dont consume the nested output, this is only required by the frontend
            // to show the reasoning thoughts, it will be returned again in the tool result
            StreamResponse::NestedAgentOutput { .. } => "".to_string(),
            StreamResponse::ParToolCall { .. } => {
                todo!(
                    "deep research currently doesn't support par tool calls"
                )
            }
            StreamResponse::ParToolResult { .. } => {
                todo!(
                    "deep research currently doesn't support par tool results"
                )
            }
        }
    }
}

#[derive(Clone)]
pub enum Model {
    Claude(Arc<ClaudeAgent>),
    Gemini(Arc<GeminiAgent>),
    DeepSeek(Arc<DeepSeekAgent>),
    OpenAI(Arc<OpenAIAgent>),
    OpenRouter(Arc<OpenRouterAgent>),
}

pub struct ReasoningLoop {
    model: Model,
    stdout: bool,
}

impl ReasoningLoop {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            stdout: true,
        }
    }

    pub async fn stream(
        &self,
        prompt: String,
        messages: Vec<Message>,
        tx: Option<Sender<StreamResponse>>,
        global_memory: Option<Arc<GraphMemory>>,
        user_id: String,
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

        Self::with_stream_channel(tx.clone(), || async {
            match &self.model {
                Model::Gemini(agent) => {
                    self.stream_gemini(agent, prompt, messages, tx).await
                }
                _ => {
                    self.stream_generic(
                        self.model.clone(),
                        prompt,
                        messages,
                        tx,
                        global_memory,
                        user_id,
                    )
                    .await
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

impl ReasoningLoop {
    // Add this new helper function
    pub async fn with_stream_channel<F, Fut, T>(
        channel: Option<Sender<StreamResponse>>,
        f: F,
    ) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        CURRENT_STREAM_CHANNEL
            .scope(RefCell::new(channel), f())
            .await
    }

    // Function to get the current stream channel
    pub async fn get_current_stream_channel() -> Option<Sender<StreamResponse>>
    {
        CURRENT_STREAM_CHANNEL
            .try_with(|c| c.borrow().clone())
            .unwrap_or_default()
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
}
