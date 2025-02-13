use actix_ws::{Message, Session};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info};

use crate::redis_subscriber::RedisSubscriber;

#[derive(Deserialize)]
struct SubscribeMessage {
    action: String,
    mints: Vec<String>,
}

#[derive(Serialize)]
struct ErrorMessage {
    error: String,
}

pub struct AppState {
    pub redis_subscriber: Arc<RedisSubscriber>,
}

pub async fn handle_ws_connection(
    mut session: Session,
    mut msg_stream: impl Stream<Item = Result<Message, actix_ws::ProtocolError>> + Unpin,
    redis_subscriber: Arc<RedisSubscriber>,
) {
    info!("WebSocket connection established");

    // Get a new broadcast receiver
    let mut redis_rx = redis_subscriber.subscribe();

    // Track subscribed mints
    let mut subscribed_mints: Vec<String> = Vec::new();
    let mut subscribe_all = false;

    loop {
        tokio::select! {
            // Handle WebSocket messages
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    Message::Close(reason) => {
                        info!("WebSocket connection closed: {:?}", reason);
                        break;
                    }
                    Message::Ping(bytes) => {
                        if let Err(e) = session.pong(&bytes).await {
                            error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Message::Text(text) => {
                        match serde_json::from_str::<SubscribeMessage>(&text) {
                            Ok(subscribe_msg) => {
                                if subscribe_msg.action == "subscribe" {
                                    // Check for wildcard subscription
                                    subscribe_all = subscribe_msg.mints.iter().any(|m| m == "*");
                                    subscribed_mints = if subscribe_all {
                                        Vec::new() // Clear specific subscriptions if wildcard is used
                                    } else {
                                        subscribe_msg.mints
                                    };
                                    info!(
                                        "Updated subscriptions: {}",
                                        if subscribe_all {
                                            "all mints (wildcard)".to_string()
                                        } else {
                                            format!("specific mints: {:?}", subscribed_mints)
                                        }
                                    );
                                }
                            }
                            Err(e) => {
                                let error_msg = ErrorMessage {
                                    error: format!("Invalid message format: {}", e),
                                };
                                if let Err(e) = session.text(serde_json::to_string(&error_msg).unwrap()).await {
                                    error!("Failed to send error message: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            Ok(msg) = redis_rx.recv() => {
                if let Ok(json) = serde_json::from_str::<Value>(&msg) {
                    if let Some(mint) = json.get("pubkey").and_then(|m| m.as_str()) {
                        if subscribe_all || subscribed_mints.iter().any(|m| m == mint) {
                            if let Err(e) = session.text(msg).await {
                                error!("Failed to send message: {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            else => break,
        }
    }

    let _ = session.close(None).await;
}
