use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use super::state::EngineMessage;
use crate::engine::api::PipelineParams;
use crate::engine::api::WirePipeline;
use crate::engine::pipeline::Pipeline;
use crate::server::state::AppState;
use tokio::sync::oneshot;

#[derive(Deserialize)]
pub struct CreatePipelineSearchParams {
    pub user_id: String,
}

pub async fn create_pipeline_internal(
    data: web::Data<AppState>,
    params: web::Query<CreatePipelineSearchParams>,
    wire: web::Json<WirePipeline>,
) -> impl Responder {
    let start = std::time::Instant::now();
    let user_id = &params.user_id;

    match data.privy.get_user_by_id(user_id).await {
        Ok(user) => {
            let user_info = data.privy.user_to_user_info(&user);

            if user_info.pubkey.is_none() {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "status": "error",
                    "message": "Wallet is required in order to create a pipeline"
                }));
            }

            metrics::counter!("pipeline_creation_attempts", 1);

            let pipeline: Pipeline = (
                wire.into_inner(),
                PipelineParams {
                    user_id: user_id.to_string(),
                    wallet_address: user_info.wallet_address.clone(),
                    pubkey: user_info.pubkey.clone(),
                },
            )
                .into();

            tracing::info!(pipeline = ?pipeline, "creating pipeline");

            // Create oneshot channel for response
            let (response_tx, response_rx) = oneshot::channel();

            // Send message to engine
            if let Err(e) = data
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
            let result =
                match tokio::time::timeout(std::time::Duration::from_secs(5), response_rx).await {
                    Ok(response) => match response {
                        Ok(Ok(id)) => {
                            metrics::counter!("pipeline_creation_success", 1);
                            HttpResponse::Created().json(serde_json::json!({
                                "status": "success",
                                "message": "Pipeline created successfully",
                                "id": id
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
        Err(e) => {
            tracing::error!("Failed to get user from Privy: {:?}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": "Invalid user ID"
            }))
        }
    }
}
