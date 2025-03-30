use crate::agents::listen::create_listen_agent_gemini;
use crate::common::spawn_with_signer;
use crate::cross_chain::agent::create_cross_chain_agent;
use crate::evm::agent::create_evm_agent;
use crate::http::middleware::verify_auth;
use crate::http::serde::deserialize_messages;
use crate::http::state::AppState;
use crate::reasoning_loop::Model;
use crate::reasoning_loop::ReasoningLoop;
use crate::reasoning_loop::StreamResponse;
use crate::signer::privy::PrivySigner;
use crate::signer::TransactionSigner;
use crate::solana::agent::create_solana_agent;
use crate::solana::agent::create_solana_agent_gemini;
use crate::solana::agent::Features;
use actix_web::{post, web, HttpRequest, Responder};
use actix_web_lab::sse;
use futures::StreamExt;
use mongodb::bson::doc;
use rig::completion::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Clone)]
pub struct ChatRequest {
    prompt: String,
    #[serde(deserialize_with = "deserialize_messages")]
    chat_history: Vec<Message>,
    #[serde(default)]
    chain: Option<String>,
    #[serde(default)]
    preamble: Option<String>,
    #[serde(default)]
    features: Option<Features>,
    model_type: Option<String>,
    #[serde(default)]
    locale: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Chat<'a> {
    pub user_id: &'a str,
    pub wallet_address: Option<&'a str>,
    pub pubkey: Option<&'a str>,
    pub chat_request: ChatRequest,
    #[serde(default)]
    pub responses: Vec<StreamResponse>,
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
    let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(1);
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        let item = match item {
            Ok(item) => item,
            Err(e) => {
                tracing::error!("Error: reading request body: {}", e);
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

    let preamble = request.preamble.clone();
    let features = request.features.clone().unwrap_or_default();

    // Select the appropriate agent based on the chain parameter and preamble
    let agent = match request.chain.as_deref() {
        #[cfg(feature = "solana")]
        Some("solana") => {
            match create_solana_agent(preamble, features.clone()).await {
                Ok(agent) => Arc::new(agent),
                Err(e) => {
                    tracing::error!(
                        "Error: failed to create Solana agent: {}",
                        e
                    );
                    let error_event = sse::Event::Data(sse::Data::new(
                        serde_json::to_string(&StreamResponse::Error(
                            format!("Failed to create Solana agent: {}", e),
                        ))
                        .unwrap(),
                    ));
                    let _ = tx.send(error_event).await;
                    return sse::Sse::from_infallible_receiver(rx);
                }
            }
        }
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
    let locale = request.locale.clone().unwrap_or("en".to_string());

    let signer: Arc<dyn TransactionSigner> = Arc::new(PrivySigner::new(
        state.privy.clone(),
        user_session.clone(),
        locale,
    ));

    // Create a channel for collecting responses - this stays put
    let (response_tx, response_rx) =
        tokio::sync::mpsc::channel::<StreamResponse>(1024);

    // Store all responses in this vec TODO move this to a separate method to be used by swarm leader calls
    // to collect the subresponses (potentailly, in case it'd be to run offline)
    let response_collector = {
        let mut rx = response_rx;
        let mongo = state.mongo.clone();
        let user_id = user_session.user_id.clone();
        let wallet_address = user_session.wallet_address.clone();
        let chat_request = request.clone();

        async move {
            let mut collected_responses = Vec::new();

            while let Some(response) = rx.recv().await {
                collected_responses.push(response);
            }

            // Only save if we have responses
            if !collected_responses.is_empty() {
                let collection = mongo.collection::<Chat>("chats");
                let chat = Chat {
                    user_id: user_id.as_str(),
                    wallet_address: wallet_address.as_deref(),
                    pubkey: None,
                    chat_request,
                    responses: join_responses(collected_responses),
                };

                match collection.insert_one(chat, None).await {
                    Ok(_) => tracing::info!(
                        "Successfully saved chat with responses to MongoDB"
                    ),
                    Err(e) => {
                        tracing::error!(
                            "Failed to save chat to MongoDB: {}",
                            e
                        )
                    }
                }
            }
        }
    };

    // Process responses in the background - don't wait for it
    tokio::spawn(response_collector);

    let preamble = request.preamble.clone();
    let model_type = request.model_type.clone().unwrap_or_default();
    // Do the main processing with the signer
    spawn_with_signer(signer, || async move {
        let reasoning_loop = if model_type == "gemini" {
            let model = if features.deep_research {
                Model::Gemini(Arc::new(create_listen_agent_gemini()))
            } else {
                Model::Gemini(Arc::new(create_solana_agent_gemini(
                    preamble.clone(),
                    features,
                )))
            };
            ReasoningLoop::new(model).with_stdout(false)
        } else {
            ReasoningLoop::new(Model::Anthropic(agent)).with_stdout(false)
        };

        // Create a channel for the reasoning loop to send responses
        let (internal_tx, mut internal_rx) =
            tokio::sync::mpsc::channel::<StreamResponse>(1024);

        // Create a separate task to handle sending responses
        let tx_clone = tx.clone();
        let response_tx_clone = response_tx.clone();
        let send_task = tokio::spawn(async move {
            while let Some(response) = internal_rx.recv().await {
                // Send to client
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

                // Send to our storage channel
                let _ = response_tx_clone.send(response).await;
            }

            // Close the response channel to signal completion
            drop(response_tx_clone);
        });

        // Make the current channel available in the global task context
        // so nested agents can access it
        crate::reasoning_loop::set_current_stream_channel(Some(
            internal_tx.clone(),
        ))
        .await;

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

    // Return the SSE stream immediately without waiting for all responses
    sse::Sse::from_infallible_receiver(rx)
}

/// helper to aggregate streamed message chunks into one and respect breaks on tool call/output
fn join_responses(
    input_responses: Vec<StreamResponse>,
) -> Vec<StreamResponse> {
    let mut output_responses = Vec::new();
    let mut message_acc = String::new();

    for response in input_responses {
        match response {
            StreamResponse::Message(message) => {
                message_acc.push_str(&message);
            }
            StreamResponse::NestedAgentOutput {
                agent_type,
                content,
            } => {
                // Pass through nested agent outputs unchanged
                if !message_acc.is_empty() {
                    output_responses
                        .push(StreamResponse::Message(message_acc));
                    message_acc = String::new();
                }
                output_responses.push(StreamResponse::NestedAgentOutput {
                    agent_type,
                    content,
                });
            }
            StreamResponse::ToolCall { id, name, params } => {
                // Only push accumulated message if it's not empty
                if !message_acc.is_empty() {
                    output_responses
                        .push(StreamResponse::Message(message_acc));
                    message_acc = String::new();
                }
                output_responses.push(StreamResponse::ToolCall {
                    id,
                    name,
                    params,
                });
            }
            StreamResponse::ToolResult { id, name, result } => {
                // Only push accumulated message if it's not empty
                if !message_acc.is_empty() {
                    output_responses
                        .push(StreamResponse::Message(message_acc));
                    message_acc = String::new();
                }
                output_responses.push(StreamResponse::ToolResult {
                    id,
                    name,
                    result,
                });
            }
            StreamResponse::Error(error) => {
                // Only push accumulated message if it's not empty
                if !message_acc.is_empty() {
                    output_responses
                        .push(StreamResponse::Message(message_acc));
                    message_acc = String::new();
                }
                output_responses.push(StreamResponse::Error(error));
            }
        }
    }

    // Add any remaining accumulated message
    if !message_acc.is_empty() {
        output_responses.push(StreamResponse::Message(message_acc));
    }

    output_responses
}
