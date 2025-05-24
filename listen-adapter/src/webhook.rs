use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct PrivyWebhookPayload {
    #[serde(rename = "type")]
    pub event: String,
    pub transaction_id: String,
    pub wallet_id: String,
    pub transaction_hash: String,
    #[serde(rename = "caip2")]
    pub chain_id: String,
}

pub async fn webhook(
    payload: web::Json<serde_json::Value>,
    state: web::Data<AppState>,
) -> HttpResponse {
    tracing::info!("Received webhook: {:?}", payload);

    let data = match serde_json::from_value::<PrivyWebhookPayload>(payload.0) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to parse webhook payload: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    // Publish the webhook payload to the transaction_updates channel
    match state.redis_client.publish_transaction_update(&data).await {
        Ok(_) => {
            tracing::info!("Passing on the webhook");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to publish webhook to Redis: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
