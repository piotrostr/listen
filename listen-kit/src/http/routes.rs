use crate::solana::signer::privy::PrivySigner;
use crate::solana::signer::{SignerContext, TransactionSigner};

use super::middleware::verify_auth;
use super::state::AppState;
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    get, post, web, Error, HttpRequest, HttpResponse, Responder,
};
use actix_web_lab::sse;
use futures_util::StreamExt;
use rig::completion::Message;
use rig::streaming::{StreamingChat, StreamingChoice};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::future::Future;
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

#[derive(Serialize)]
pub enum ServerError {
    WalletError,
    PrivyError,
}

pub async fn spawn_with_signer<F, Fut, T>(
    signer: Arc<dyn TransactionSigner>,
    f: F,
) -> tokio::task::JoinHandle<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        SignerContext::with_signer(signer, async { f().await }).await
    })
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
                serde_json::to_string("Error: unauthorized").unwrap(),
            ));
            let _ = tx.send(error_event).await;
            return sse::Sse::from_infallible_receiver(rx);
        }
    };

    let (tx, rx) = tokio::sync::mpsc::channel::<sse::Event>(32);
    let agent = state.agent.clone();
    let prompt = request.prompt.clone();
    let messages = request.chat_history.clone();

    let signer: Arc<dyn TransactionSigner> = Arc::new(PrivySigner::new(
        state.wallet_manager.clone(),
        user_session.clone(),
    ));

    spawn_with_signer(signer, || async move { // FIXME indent
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
                        tracing::debug!(tool = name, parameters = ?params, "Tool call");
                        match agent.tools.call(&name, params.to_string()).await {
                            Ok(result) => {
                                tracing::debug!(tool = name, result = ?result, "Tool call result");
                                StreamResponse::ToolCall {
                                    name: name.to_string(),
                                    result: result.to_string(),
                                }
                            }
                            Err(e) => {
                                tracing::error!(tool = name, error = ?e, "Tool call error");
                                StreamResponse::Error(format!(
                                    "Tool call failed: {}",
                                    e
                                ))
                            }
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
        }).await;

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

// this is just to check whether transaction signing through privy works it
// will get remove in the
// future and the logic incorporated in the tools as opt-in
#[get("/test_tx")]
async fn test_tx(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let user_session = match verify_auth(&req).await {
        Ok(session) => session,
        Err(_) => return Ok(HttpResponse::Unauthorized().finish()),
    };
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::message::Message;
    use solana_sdk::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_instruction::transfer;
    use solana_sdk::transaction::Transaction;
    use std::str::FromStr;

    let user =
        Pubkey::from_str(user_session.wallet_address.as_str()).unwrap();
    let tx = Transaction::new_unsigned(Message::new_with_blockhash(
        &[transfer(
            &user,
            &pubkey!("aiamaErRMjbeNmf2b8BMZWFR3ofxrnZEf2mLKp935fM"),
            100000,
        )],
        Some(&user),
        &RpcClient::new("https://api.mainnet-beta.solana.com".to_string())
            .get_latest_blockhash()
            .await
            .unwrap(),
    ));

    let signature = state
        .wallet_manager
        .sign_and_send_transaction(user_session.wallet_address, &tx)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "signature": signature,
    })))
}
