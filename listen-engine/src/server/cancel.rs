use super::state::{AppState, EngineMessage};
use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;
use tokio::sync::oneshot;
use uuid::Uuid;

use super::common::{handle_engine_response, verify_auth};

#[derive(Deserialize)]
pub struct CancelStepParams {
    pipeline_id: Uuid,
    step_id: Uuid,
}

pub async fn cancel_pipeline(
    state: Data<AppState>,
    req: HttpRequest,
    path: Path<Uuid>,
) -> impl Responder {
    let pipeline_id = path.into_inner();

    // Authenticate user
    let user = match verify_auth(&state, &req).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    // Create channel for response
    let (response_tx, response_rx) = oneshot::channel();

    // Send cancel message to engine
    if let Err(e) = state
        .engine_bridge_tx
        .send(EngineMessage::CancelPipeline {
            user_id: user.user_id.clone(),
            pipeline_id,
            response_tx,
        })
        .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("Failed to communicate with engine: {}", e)
        }));
    }

    handle_engine_response(response_rx, "Pipeline cancelled successfully").await
}

pub async fn cancel_step(
    state: Data<AppState>,
    req: HttpRequest,
    params: Path<CancelStepParams>,
) -> impl Responder {
    let CancelStepParams {
        pipeline_id,
        step_id,
    } = params.into_inner();

    // Authenticate user
    let user = match verify_auth(&state, &req).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    // Create channel for response
    let (response_tx, response_rx) = oneshot::channel();

    // Send cancel step message to engine
    if let Err(e) = state
        .engine_bridge_tx
        .send(EngineMessage::CancelStep {
            user_id: user.user_id.clone(),
            pipeline_id,
            step_id,
            response_tx,
        })
        .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "message": format!("Failed to communicate with engine: {}", e)
        }));
    }

    handle_engine_response(response_rx, "Step cancelled successfully").await
}
