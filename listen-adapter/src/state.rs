use std::sync::Arc;

use crate::redis_subscriber::RedisSubscriber;

#[derive(Clone)]
pub struct AppState {
    pub redis_subscriber: Arc<RedisSubscriber>,
}
