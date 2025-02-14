use std::sync::Arc;

use crate::db::ClickhouseDb;
use crate::redis_client::RedisClient;
use crate::redis_subscriber::RedisSubscriber;

#[derive(Clone)]
pub struct AppState {
    pub redis_subscriber: Arc<RedisSubscriber>,
    pub redis_client: Arc<RedisClient>,
    pub clickhouse_db: Arc<ClickhouseDb>,
}
