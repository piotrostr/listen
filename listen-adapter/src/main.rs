use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::sync::Arc;
use tracing::info;

use listen_adapter::{
    redis_subscriber::RedisSubscriber, state::AppState, websocket::handle_ws_connection,
};

async fn create_redis_subscriber(redis_url: &str) -> anyhow::Result<Arc<RedisSubscriber>> {
    let subscriber = RedisSubscriber::new(redis_url)?;
    subscriber.start_listening("price_updates").await?;

    Ok(Arc::new(subscriber))
}

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn WebSocket handler
    actix_web::rt::spawn(handle_ws_connection(
        session,
        msg_stream,
        state.redis_subscriber.clone(),
    ));

    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "6968".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    let redis_subscriber = create_redis_subscriber(&redis_url)
        .await
        .expect("Failed to create Redis subscriber");

    let app_state = AppState { redis_subscriber };

    info!("Starting WebSocket server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/ws", web::get().to(ws_route))
    })
    .bind((host, port))?
    .run()
    .await
}
