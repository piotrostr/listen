use actix_ws::{Message, Session};
use futures::{Stream, StreamExt};
use std::sync::Arc;
use tracing::{error, info};

use crate::redis_subscriber::RedisSubscriber;

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
                        info!("Received message: {}", text);
                    }
                    _ => {}
                }
            }

            // Handle Redis messages
            Ok(msg) = redis_rx.recv() => {
                if let Err(e) = session.text(msg).await {
                    error!("Failed to send message: {}", e);
                    break;
                }
            }

            else => break,
        }
    }

    let _ = session.close(None).await;
}
