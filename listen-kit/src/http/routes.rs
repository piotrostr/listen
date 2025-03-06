use super::middleware::verify_auth;
use super::serde::deserialize_messages;
use super::state::AppState;
use crate::common::spawn_with_signer;
use crate::cross_chain::agent::create_cross_chain_agent;
use crate::evm::agent::create_evm_agent;
use crate::reasoning_loop::ReasoningLoop;
use crate::reasoning_loop::StreamResponse;
use crate::signer::privy::PrivySigner;
use crate::signer::TransactionSigner;
use crate::solana::agent::create_solana_agent;
use actix_web::{
    get, post, web, Error, HttpRequest, HttpResponse, Responder,
};
use actix_web_lab::sse;
use anyhow::Result;
use futures::StreamExt;
use rig::completion::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct ChatRequest {
    prompt: String,
    #[serde(deserialize_with = "deserialize_messages")]
    chat_history: Vec<Message>,
    #[serde(default)]
    chain: Option<String>,
    #[serde(default)]
    preamble: Option<String>,
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
    mut body: web::Payload,
) -> impl Responder {
    // Extract and collect the body
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        let item = match item {
            Ok(item) => item,
            Err(e) => {
                tracing::error!("Error: reading request body: {}", e);
                let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(1);
                let error_event = sse::Event::Data(sse::Data::new(
                    serde_json::to_string(&StreamResponse::Error(format!(
                        "Error reading request body: {}",
                        e
                    )))
                    .unwrap(),
                ));
                let _ = tx.send(error_event).await;
                return sse::Sse::from_infallible_receiver(rx);
            }
        };
        bytes.extend_from_slice(&item);
    }

    // Log the raw request body
    // println!("Raw request body: {}", String::from_utf8_lossy(&bytes));

    // Deserialize into ChatRequest
    let request: ChatRequest = match serde_json::from_slice(&bytes) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!("Error: deserializing request: {}", e);
            let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(1);
            let error_event = sse::Event::Data(sse::Data::new(
                serde_json::to_string(&StreamResponse::Error(format!(
                    "Error deserializing request: {}",
                    e
                )))
                .unwrap(),
            ));
            let _ = tx.send(error_event).await;
            return sse::Sse::from_infallible_receiver(rx);
        }
    };

    let user_session = match verify_auth(&req).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Error: unauthorized: {}", e);
            let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(1);
            let error_event = sse::Event::Data(sse::Data::new(
                serde_json::to_string(&StreamResponse::Error(format!(
                    "Error: unauthorized: {}",
                    e
                )))
                .unwrap(),
            ));
            let _ = tx.send(error_event).await;
            return sse::Sse::from_infallible_receiver(rx);
        }
    };

    let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(1024);

    let preamble = request.preamble.clone();

    // Select the appropriate agent based on the chain parameter and preamble
    let agent = match request.chain.as_deref() {
        #[cfg(feature = "solana")]
        Some("solana") => match create_solana_agent(preamble).await {
            Ok(agent) => Arc::new(agent),
            Err(e) => {
                tracing::error!(
                    "Error: failed to create Solana agent: {}",
                    e
                );
                let error_event = sse::Event::Data(sse::Data::new(
                    serde_json::to_string(&StreamResponse::Error(format!(
                        "Failed to create Solana agent: {}",
                        e
                    )))
                    .unwrap(),
                ));
                let _ = tx.send(error_event).await;
                return sse::Sse::from_infallible_receiver(rx);
            }
        },
        #[cfg(feature = "evm")]
        Some("evm") => match create_evm_agent(preamble).await {
            Ok(agent) => Arc::new(agent),
            Err(e) => {
                tracing::error!("Error: failed to create EVM agent: {}", e);
                let error_event = sse::Event::Data(sse::Data::new(
                    serde_json::to_string(&StreamResponse::Error(format!(
                        "Failed to create EVM agent: {}",
                        e
                    )))
                    .unwrap(),
                ));
                let _ = tx.send(error_event).await;
                return sse::Sse::from_infallible_receiver(rx);
            }
        },
        Some("omni") => match create_cross_chain_agent(preamble).await {
            Ok(agent) => Arc::new(agent),
            Err(e) => {
                tracing::error!(
                    "Error: failed to create cross-chain agent: {}",
                    e
                );
                let error_event = sse::Event::Data(sse::Data::new(
                    serde_json::to_string(&StreamResponse::Error(format!(
                        "Failed to create cross-chain agent: {}",
                        e
                    )))
                    .unwrap(),
                ));
                let _ = tx.send(error_event).await;
                return sse::Sse::from_infallible_receiver(rx);
            }
        },
        Some(chain) => {
            tracing::error!("Error: unsupported chain: {}", chain);
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
            tracing::error!("Chain parameter is required");
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

    let signer: Arc<dyn TransactionSigner> =
        Arc::new(PrivySigner::new(state.privy.clone(), user_session.clone()));

    spawn_with_signer(signer, || async move {
        let reasoning_loop = ReasoningLoop::new(agent).with_stdout(false);

        // Create a channel for the reasoning loop to send responses
        let (internal_tx, mut internal_rx) = tokio::sync::mpsc::channel(1024);

        // Create a separate task to handle sending responses
        let tx_clone = tx.clone();
        let send_task = tokio::spawn(async move {
            while let Some(response) = internal_rx.recv().await {
                if tx_clone
                    .send(sse::Event::Data(sse::Data::new(
                        serde_json::to_string(&response).unwrap(),
                    )))
                    .await
                    .is_err()
                {
                    tracing::error!("Error: failed to send response");
                    break;
                }
            }
        });

        // Run the reasoning loop in the current task (with signer context)
        let loop_result = reasoning_loop
            .stream(prompt, messages, Some(internal_tx))
            .await;

        // Wait for the send task to complete
        let _ = send_task.await;

        // Check if the reasoning loop completed successfully
        if let Err(e) = loop_result {
            tracing::error!("Error: reasoning loop failed: {}", e);
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
