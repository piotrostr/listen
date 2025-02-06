use super::middleware::verify_auth;
use super::state::AppState;
use crate::common::spawn_with_signer;
use crate::reasoning_loop::LoopResponse;
use crate::reasoning_loop::ReasoningLoop;
use crate::signer::privy::PrivySigner;
use crate::signer::TransactionSigner;
use actix_web::{
    get, post, web, Error, HttpRequest, HttpResponse, Responder,
};
use actix_web_lab::sse;
use anyhow::Result;
use rig::completion::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize)]
pub struct ChatRequest {
    prompt: String,
    chat_history: Vec<Message>,
    #[serde(default)]
    chain: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "content")]
pub enum StreamResponse {
    Message(String),
    ToolCall { name: String, result: String },
    Error(String),
}

#[derive(Serialize)]
pub enum ServerError {
    WalletError,
    PrivyError,
    ChainNotSupported,
}

#[post("/stream")]
async fn stream(
    req: HttpRequest,
    state: web::Data<AppState>,
    request: web::Json<ChatRequest>,
) -> impl Responder {
    let user_session = match verify_auth(&req).await {
        Ok(s) => s,
        Err(_) => {
            let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(1);
            let error_event = sse::Event::Data(sse::Data::new(
                serde_json::to_string(&StreamResponse::Error(
                    "Error: unauthorized".to_string(),
                ))
                .unwrap(),
            ));
            let _ = tx.send(error_event).await;
            return sse::Sse::from_infallible_receiver(rx);
        }
    };

    let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(32);

    // Select the appropriate agent based on the chain parameter
    let agent = match request.chain.as_deref() {
        #[cfg(feature = "solana")]
        Some("solana") => state.solana_agent.clone(),
        #[cfg(feature = "evm")]
        Some("evm") => state.evm_agent.clone(),
        Some("omni") => state.omni_agent.clone(),
        Some(chain) => {
            let error_event = sse::Event::Data(sse::Data::new(
                serde_json::to_string(&StreamResponse::Error(format!(
                    "Unsupported chain: {}",
                    chain
                )))
                .unwrap(),
            ));
            let _ = tx.send(error_event).await;
            return sse::Sse::from_infallible_receiver(rx);
        }
        None => {
            let error_event = sse::Event::Data(sse::Data::new(
                serde_json::to_string(&StreamResponse::Error(
                    "Chain parameter is required".to_string(),
                ))
                .unwrap(),
            ));
            let _ = tx.send(error_event).await;
            return sse::Sse::from_infallible_receiver(rx);
        }
    };

    let prompt = request.prompt.clone();
    let messages = request.chat_history.clone();

    let signer: Arc<dyn TransactionSigner> = Arc::new(PrivySigner::new(
        state.wallet_manager.clone(),
        user_session.clone(),
    ));

    spawn_with_signer(signer, || async move {
        let reasoning_loop = ReasoningLoop::new(agent).with_stdout(false);

        let mut initial_messages = messages;
        initial_messages.push(Message {
            role: "user".to_string(),
            content: prompt,
        });

        // Create a channel for the reasoning loop to send responses
        let (internal_tx, mut internal_rx) = tokio::sync::mpsc::channel(32);

        // Create a separate task to handle sending responses
        let tx_clone = tx.clone();
        let send_task = tokio::spawn(async move {
            while let Some(response) = internal_rx.recv().await {
                let stream_response = match response {
                    LoopResponse::Message(text) => {
                        StreamResponse::Message(text)
                    }
                    LoopResponse::ToolCall { name, result } => {
                        StreamResponse::ToolCall { name, result }
                    }
                };

                if tx_clone
                    .send(sse::Event::Data(sse::Data::new(
                        serde_json::to_string(&stream_response).unwrap(),
                    )))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        // Run the reasoning loop in the current task (with signer context)
        let loop_result = reasoning_loop
            .stream(initial_messages, Some(internal_tx))
            .await;

        // Wait for the send task to complete
        let _ = send_task.await;

        // Check if the reasoning loop completed successfully
        if let Err(e) = loop_result {
            let _ = tx
                .send(sse::Event::Data(sse::Data::new(
                    serde_json::to_string(&StreamResponse::Error(
                        e.to_string(),
                    ))
                    .unwrap(),
                )))
                .await;
        }

        Ok(())
    })
    .await;

    sse::Sse::from_infallible_receiver(rx)
        .with_keep_alive(Duration::from_secs(15))
        .with_retry_duration(Duration::from_secs(10))
}

#[get("/healthz")]
async fn healthz() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[get("/auth")]
async fn auth(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user_session = match verify_auth(&req).await {
        Ok(session) => session,
        Err(e) => {
            return Ok(HttpResponse::Unauthorized()
                .json(json!({ "error": e.to_string() })))
        }
    };

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "wallet_address": user_session.wallet_address,
    })))
}
