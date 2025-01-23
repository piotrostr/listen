#[cfg(feature = "http")]
use {
    actix_cors::Cors,
    actix_web::{get, post, web, App, HttpServer, Responder},
    actix_web_lab::sse,
    futures_util::StreamExt,
    rig::agent::Agent,
    rig::completion::Message,
    rig::providers::anthropic::completion::CompletionModel,
    rig::streaming::{StreamingChat, StreamingChoice},
    serde::{Deserialize, Serialize},
    serde_json::json,
    std::sync::Arc,
    std::time::Duration,
};

#[cfg(feature = "http")]
#[derive(Deserialize)]
pub struct ChatRequest {
    prompt: String,
    chat_history: Vec<Message>,
}

#[cfg(feature = "http")]
#[derive(Serialize)]
#[serde(tag = "type", content = "content")]
pub enum StreamResponse {
    Message(String),
    ToolCall { name: String, result: String },
    Error(String),
}

#[cfg(feature = "http")]
pub struct AppState {
    agent: Arc<Agent<CompletionModel>>,
}

#[cfg(feature = "http")]
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

#[cfg(feature = "http")]
#[get("/health")]
async fn health_check() -> impl Responder {
    web::Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[cfg(feature = "http")]
pub async fn run_server(agent: Agent<CompletionModel>) -> std::io::Result<()> {
    use actix_web::middleware::{Compress, Logger};

    let state = web::Data::new(AppState {
        agent: Arc::new(agent),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .service(stream)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
