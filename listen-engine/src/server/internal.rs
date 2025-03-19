use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use super::create::create_pipeline_common;
use crate::engine::api::{PipelineParams, WirePipeline};
use crate::server::state::AppState;

#[derive(Deserialize)]
pub struct CreatePipelineRequest {
    pub user_id: String,
    pub pipeline: WirePipeline,
}

pub async fn create_pipeline_internal(
    data: web::Data<AppState>,
    json: web::Json<CreatePipelineRequest>,
) -> impl Responder {
    let request = json.into_inner();
    match data.privy.get_user_by_id(&request.user_id).await {
        Ok(user) => {
            let user_info = data.privy.user_to_user_info(&user);

            let pipeline_params = PipelineParams {
                user_id: request.user_id.to_string(),
                wallet_address: user_info.wallet_address.clone(),
                pubkey: user_info.pubkey.clone(),
            };

            create_pipeline_common(data, request.pipeline, pipeline_params).await
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
