use super::state::{AppState, EngineMessage};
use crate::{
    engine::{
        api::{PipelineParams, WirePipeline},
        pipeline::Pipeline,
    },
    server::common::{handle_engine_response, verify_auth},
};
use actix_web::{
    web::{self, Data},
    HttpRequest, HttpResponse, Responder,
};
use tokio::sync::oneshot;

// New function for common pipeline creation logic
pub async fn create_pipeline_common(
    state: Data<AppState>,
    wire: WirePipeline,
    pipeline_params: PipelineParams,
) -> HttpResponse {
    let start = std::time::Instant::now();

    metrics::counter!("pipeline_creation_attempts", 1);

    if pipeline_params.pubkey.is_none() && pipeline_params.wallet_address.is_none() {
        metrics::counter!("pipeline_creation_errors_no_wallet", 1);
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": "Wallet is required in order to create a pipeline"
        }));
    }

    let pipeline: Pipeline = (wire, pipeline_params).into();

    tracing::info!(pipeline = ?pipeline, "creating pipeline");

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
    let result = handle_engine_response(response_rx, "Pipeline created successfully").await;

    metrics::histogram!("pipeline_creation_duration", start.elapsed());

    result
}

pub async fn create_pipeline(
    state: Data<AppState>,
    req: HttpRequest,
    wire: web::Json<WirePipeline>,
) -> impl Responder {
    let user = match verify_auth(&state, &req).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let pipeline_params = PipelineParams {
        user_id: user.user_id,
        wallet_address: user.wallet_address.clone(),
        pubkey: user.pubkey.clone(),
    };

    create_pipeline_common(state, wire.into_inner(), pipeline_params).await
}
