use actix_cors::Cors;
use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    engine::{
        api::{PipelineParams, WirePipeline},
        pipeline::Pipeline,
        Engine, EngineError,
    },
    metrics::metrics_handler,
};
use privy::{config::PrivyConfig, Privy};

#[derive(Debug)]
pub enum EngineMessage {
    AddPipeline {
        pipeline: Pipeline,
        response_tx: oneshot::Sender<Result<(), EngineError>>,
    },
    GetPipeline {
        user_id: String,
        pipeline_id: Uuid,
        response_tx: oneshot::Sender<Result<Pipeline, EngineError>>,
    },
    DeletePipeline {
        user_id: String,
        pipeline_id: Uuid,
        response_tx: oneshot::Sender<Result<(), EngineError>>,
    },
    GetAllPipelinesByUser {
        user_id: String,
        response_tx: oneshot::Sender<Result<Vec<Pipeline>, EngineError>>,
    },
}

pub struct AppState {
    engine_bridge_tx: mpsc::Sender<EngineMessage>,
    privy: Arc<Privy>,
}

pub async fn run() -> std::io::Result<()> {
    let (tx, rx) = mpsc::channel(1000);
    tracing::info!("Created channel with capacity 1000");
    let mut engine = match Engine::from_env().await {
        Ok(engine) => engine,
        Err(e) => {
            tracing::error!("Failed to create engine: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create engine",
            ));
        }
    };

    // Create a shutdown signal handler
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
    let shutdown_tx_clone = shutdown_tx.clone();

    // Set up ctrl-c handler
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            let _ = shutdown_tx_clone.send(()).await;
        }
    });

    let privy = Arc::new(Privy::new(PrivyConfig::from_env().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to create privy config")
    })?));

    // Main application server with metrics endpoint
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                engine_bridge_tx: tx.clone(),
                privy: privy.clone(),
            }))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .expose_headers(["content-type", "authorization"])
                    .max_age(3600)
                    .allowed_header(actix_web::http::header::CONTENT_TYPE)
                    .allowed_header(actix_web::http::header::AUTHORIZATION),
            )
            .wrap(middleware::Logger::default())
            .route("/healthz", web::get().to(healthz))
            .route("/pipeline", web::post().to(create_pipeline))
            .route("/pipelines", web::get().to(get_pipelines))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind(("0.0.0.0", 6966))?
    .run();

    tokio::select! {
        result = server => {
            let _ = shutdown_tx.send(()).await;
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        result = engine.run(rx) => {
            let _ = shutdown_tx.send(()).await;
            if let Err(e) = result {
                tracing::error!("Engine error: {}", e);
            }
        }
        _ = shutdown_rx.recv() => {
            tracing::info!("Shutdown signal received, starting graceful shutdown");
        }
    }

    tracing::info!("Server shutdown complete");
    Ok(())
}

async fn healthz() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}

async fn get_pipelines(state: Data<AppState>, req: HttpRequest) -> impl Responder {
    let auth_token = match req.headers().get("authorization") {
        Some(auth_token) => auth_token.to_str().unwrap(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Authorization header is required"
            }));
        }
    };
    let auth_token = auth_token.split(" ").nth(1).unwrap();

    let user = match state
        .privy
        .authenticate_user(auth_token)
        .await
        .map_err(|_| HttpResponse::Unauthorized())
    {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Unauthorized"
            }));
        }
    };

    println!(
        "user: {}, {}, {}",
        user.user_id, user.wallet_address, user.pubkey
    );

    let (response_tx, response_rx) = oneshot::channel();
    tracing::debug!("Sending GetAllPipelinesByUser message to engine");
    match state
        .engine_bridge_tx
        .send(EngineMessage::GetAllPipelinesByUser {
            user_id: user.user_id.clone(),
            response_tx,
        })
        .await
    {
        Ok(_) => tracing::debug!("Successfully sent message to engine"),
        Err(e) => {
            tracing::error!("Failed to send message to engine: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Engine communication error"
            }));
        }
    };

    tracing::debug!("Waiting for response from engine");
    let pipelines = match response_rx.await {
        Ok(Ok(pipelines)) => {
            tracing::debug!("Received {} pipelines from engine", pipelines.len());
            pipelines
        }
        Ok(Err(e)) => {
            tracing::error!("Engine error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to get pipelines: {}", e)
            }));
        }
        Err(e) => {
            tracing::error!("Channel closed: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Internal communication error"
            }));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "pipelines": pipelines
    }))
}

async fn create_pipeline(
    state: Data<AppState>,
    req: HttpRequest,
    wire: web::Json<WirePipeline>,
) -> impl Responder {
    let start = std::time::Instant::now();

    let auth_token = match req.headers().get("authorization") {
        Some(auth_token) => auth_token.to_str().unwrap(),
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Authorization header is required"
            }));
        }
    };
    let auth_token = auth_token.split(" ").nth(1).unwrap();

    let user = match state
        .privy
        .authenticate_user(auth_token)
        .await
        .map_err(|_| HttpResponse::Unauthorized())
    {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "status": "error",
                "message": "Unauthorized"
            }));
        }
    };

    metrics::counter!("pipeline_creation_attempts", 1);

    let pipeline: Pipeline = (
        wire.into_inner(),
        PipelineParams {
            user_id: user.user_id,
            wallet_address: user.wallet_address,
            pubkey: user.pubkey,
        },
    )
        .into();

    println!("Pipeline: {:#?}", pipeline);

    // Create oneshot channel for response
    let (response_tx, response_rx) = oneshot::channel();

    // Send message to engine
    if let Err(e) = state
        .engine_bridge_tx
        .send(EngineMessage::AddPipeline {
            pipeline,
            response_tx,
        })
        .await
    {
        metrics::counter!("pipeline_creation_errors", 1);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("Failed to communicate with engine: {}", e)
        }));
    }

    // Wait for response with timeout
    let result = match tokio::time::timeout(std::time::Duration::from_secs(5), response_rx).await {
        Ok(response) => match response {
            Ok(Ok(_)) => {
                metrics::counter!("pipeline_creation_success", 1);
                HttpResponse::Created().json(serde_json::json!({
                    "status": "success",
                    "message": "Pipeline created successfully"
                }))
            }
            Ok(Err(e)) => {
                metrics::counter!("pipeline_creation_errors", 1);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to create pipeline: {}", e)
                }))
            }
            Err(e) => {
                metrics::counter!("pipeline_creation_errors", 1);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to receive response from engine: {}", e)
                }))
            }
        },
        Err(_) => {
            metrics::counter!("pipeline_creation_errors", 1);
            HttpResponse::GatewayTimeout().json(serde_json::json!({
                "status": "error",
                "message": "Pipeline creation timed out"
            }))
        }
    };

    metrics::histogram!("pipeline_creation_duration", start.elapsed());
    result
}
