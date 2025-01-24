use actix_web::{get, post, web, Responder};
use actix_web_lab::sse;
use futures_util::StreamExt;
use rig::agent::Agent;
use rig::completion::Message;
use rig::providers::anthropic::completion::CompletionModel;
use rig::streaming::{StreamingChat, StreamingChoice};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize)]
pub struct ChatRequest {
    prompt: String,
    chat_history: Vec<Message>,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "content")]
pub enum StreamResponse {
    Message(String),
    ToolCall { name: String, result: String },
    Error(String),
}

pub struct AppState {
    agent: Arc<Agent<CompletionModel>>,
}

impl AppState {
    pub fn new(agent: Agent<CompletionModel>) -> Self {
        Self {
            agent: Arc::new(agent),
        }
    }
}

#[post("/stream")]
async fn stream(
    state: web::Data<AppState>,
    request: web::Json<ChatRequest>,
) -> impl Responder {
    let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(32);
    let agent = state.agent.clone();
    let prompt = request.prompt.clone();
    let messages = request.chat_history.clone();

    tokio::spawn(async move {
        let mut stream = match agent.stream_chat(&prompt, messages).await {
            Ok(s) => s,
            Err(e) => {
                let _ = tx
                    .send(sse::Event::Data(sse::Data::new(
                        serde_json::to_string(&StreamResponse::Error(
                            e.to_string(),
                        ))
                        .unwrap(),
                    )))
                    .await;
                return;
            }
        };

        while let Some(chunk) = stream.next().await {
            let response = match chunk {
                Ok(StreamingChoice::Message(text)) => {
                    StreamResponse::Message(text)
                }
                Ok(StreamingChoice::ToolCall(name, _, params)) => {
                    match agent.tools.call(&name, params.to_string()).await {
                        Ok(result) => StreamResponse::ToolCall {
                            name: name.to_string(),
                            result: result.to_string(),
                        },
                        Err(e) => StreamResponse::Error(format!(
                            "Tool call failed: {}",
                            e
                        )),
                    }
                }
                Err(e) => StreamResponse::Error(e.to_string()),
            };

            if tx
                .send(sse::Event::Data(sse::Data::new(
                    serde_json::to_string(&response).unwrap(),
                )))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    sse::Sse::from_infallible_receiver(rx)
        .with_keep_alive(Duration::from_secs(15))
        .with_retry_duration(Duration::from_secs(10))
}

#[get("/health")]
async fn health_check() -> impl Responder {
    web::Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
