use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize, Serialize)]
pub struct PrivyWebhookPayload {
    pub event: String,
    pub transaction_id: String,
    pub wallet_id: String,
    pub transaction_hash: String,
    pub chain_id: String,
}

pub async fn webhook(
    payload: web::Json<PrivyWebhookPayload>,
    state: web::Data<AppState>,
) -> HttpResponse {
    tracing::debug!("Received webhook: {:?}", payload);

    // Publish the webhook payload to the transaction_updates channel
    match state
        .redis_client
        .publish_transaction_update(&payload.0)
        .await
    {
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
