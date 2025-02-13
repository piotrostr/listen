use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use tracing::info;

use listen_adapter::{
    redis_subscriber::create_redis_subscriber, routes::ws_route, state::AppState,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

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
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .route("/ws", web::get().to(ws_route))
    })
    .bind((host, port))?
    .run()
    .await
}
