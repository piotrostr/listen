use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] actix_ws::ProtocolError),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AdapterError>;
