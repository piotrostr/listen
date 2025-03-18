use super::state::{AppState, EngineMessage};
use actix_web::{web::Data, HttpRequest, HttpResponse, Responder};
use tokio::sync::oneshot;

pub async fn get_pipelines(state: Data<AppState>, req: HttpRequest) -> impl Responder {
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
