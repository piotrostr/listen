use actix_ws::{Message, Session};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::redis_subscriber::RedisSubscriber;

#[derive(Debug, Deserialize)]
struct SubscribeMessage {
    action: String,
    #[serde(default)]
    mints: Vec<String>,
    #[serde(default)]
    wallet_ids: Vec<String>,
}

#[derive(Serialize)]
struct ErrorMessage {
    error: String,
}

pub async fn handle_ws_connection(
    mut session: Session,
    mut msg_stream: impl Stream<Item = Result<Message, actix_ws::ProtocolError>> + Unpin,
    redis_subscriber: Arc<RedisSubscriber>,
) {
    info!("WebSocket connection established");

    // Get broadcast receivers for both channels
    let mut price_rx = redis_subscriber.subscribe_prices();
    let mut tx_rx = redis_subscriber.subscribe_transactions();

    // Track subscriptions using HashSet for deduplication
    let mut subscribed_mints: HashSet<String> = HashSet::new();
    let mut subscribed_wallets: HashSet<String> = HashSet::new();
    let mut subscribe_all_prices = false;

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
                                    // Handle price subscriptions if mints are provided
                                    if !subscribe_msg.mints.is_empty() {
                                        if subscribe_msg.mints.iter().any(|m| m == "*") {
                                            subscribe_all_prices = true;
                                            subscribed_mints.clear();
                                            debug!("Subscribed to all price updates");
                                        } else {
                                            // Add new mints to existing subscriptions
                                            subscribed_mints.extend(subscribe_msg.mints);
                                            debug!("Updated price subscriptions: {:?}", subscribed_mints);
                                        }
                                    }

                                    // Handle transaction subscriptions if wallet_ids are provided
                                    if !subscribe_msg.wallet_ids.is_empty() {
                                        subscribed_wallets.extend(subscribe_msg.wallet_ids);
                                        debug!("Updated transaction subscriptions: {:?}", subscribed_wallets);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse message: {}, content: {}", e, text);
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

            // Handle price updates
            Ok(msg) = price_rx.recv() => {
                if let Ok(json) = serde_json::from_str::<Value>(&msg) {
                    if let Some(mint) = json.get("pubkey").and_then(|m| m.as_str()) {
                        if subscribe_all_prices || subscribed_mints.contains(mint) {
                            if let Err(e) = session.text(msg).await {
                                error!("Failed to send price message: {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            // Handle transaction updates
            Ok(msg) = tx_rx.recv() => {
                if let Ok(json) = serde_json::from_str::<Value>(&msg) {
                    if let Some(wallet_id) = json.get("wallet_id").and_then(|w| w.as_str()) {
                        if subscribed_wallets.contains(wallet_id) {
                            if let Err(e) = session.text(msg).await {
                                error!("Failed to send transaction message: {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            else => break,
        }
    }

    info!("WebSocket connection closed - cleaning up subscriptions");
    let _ = session.close(None).await;
}
