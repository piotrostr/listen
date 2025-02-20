// TODO! this should be a listen-redis create (the base) and each tenant can add
// their own commands to proc
use crate::engine::pipeline::Pipeline;
use anyhow::Result;
use bb8_redis::{
    bb8::{self, PooledConnection},
    redis::{cmd, pipe},
    RedisConnectionManager,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

const PIPELINE_BATCH_SIZE: usize = 1000;

pub struct RedisClient {
    pool: bb8::Pool<RedisConnectionManager>,
}

#[derive(Debug, thiserror::Error)]
pub enum RedisClientError {
    #[error("[Redis] Failed to create connection manager: {0}")]
    CreateConnectionManagerError(#[from] bb8_redis::redis::RedisError),
    #[error("[Redis] Failed to connect: {0}")]
    GetConnectionError(#[from] bb8::RunError<bb8_redis::redis::RedisError>),
    #[error("[Redis] Failed to serialize: {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("[Redis] Failed to deserialize: {0}")]
    DeserializeError(serde_json::Error),
    #[error("[Redis] Redis error: {0}")]
    RedisError(bb8_redis::redis::RedisError),
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self, RedisClientError> {
        let manager =
            RedisConnectionManager::new(redis_url).map_err(RedisClientError::RedisError)?;

        let pool = bb8::Pool::builder()
            .max_size(16)
            .min_idle(Some(4))
            .build(manager)
            .await
            .map_err(RedisClientError::CreateConnectionManagerError)?;

        Ok(Self { pool })
    }

    pub async fn get_connection(
        &self,
    ) -> Result<PooledConnection<'_, RedisConnectionManager>, RedisClientError> {
        self.pool
            .get()
            .await
            .map_err(RedisClientError::GetConnectionError)
    }

    async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), RedisClientError> {
        let mut conn = self.pool.get().await?;
        let serialized = serde_json::to_string(value)?;

        let _: () = cmd("SET")
            .arg(key)
            .arg(serialized)
            .query_async(&mut *conn)
            .await?;

        Ok(())
    }

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, RedisClientError> {
        let mut conn = self.pool.get().await?;

        let json_str: Option<String> = cmd("GET").arg(key).query_async(&mut *conn).await?;

        match json_str {
            Some(json_str) => Ok(Some(serde_json::from_str(&json_str)?)),
            None => Ok(None),
        }
    }

    pub async fn get_pipeline(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Option<Pipeline>, RedisClientError> {
        self.get(&format!("pipeline:{}:{}", user_id, id)).await
    }

    pub async fn save_pipeline(&self, pipeline: &Pipeline) -> Result<(), RedisClientError> {
        self.set(
            &format!("pipeline:{}:{}", pipeline.user_id, pipeline.id),
            pipeline,
        )
        .await
    }

    pub async fn get_all_pipelines_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<Pipeline>, RedisClientError> {
        let mut conn = self.pool.get().await?;

        let keys: Vec<String> = cmd("KEYS")
            .arg(format!("pipeline:{}:*", user_id))
            .query_async(&mut *conn)
            .await?;

        println!("keys: {:#?}", keys);

        let mut pipe = pipe();
        for key in &keys {
            pipe.get(key);
        }

        let results: Vec<Option<String>> = pipe.query_async(&mut *conn).await?;

        let mut pipelines = Vec::with_capacity(results.len());
        for json_str in results.into_iter().flatten() {
            match serde_json::from_str(&json_str) {
                Ok(pipeline) => pipelines.push(pipeline),
                Err(e) => warn!("Failed to deserialize pipeline: {}", e),
            }
        }

        Ok(pipelines)
    }

    pub async fn get_all_pipelines(&self) -> Result<Vec<Pipeline>, RedisClientError> {
        let mut conn = self.pool.get().await?;

        let keys: Vec<String> = cmd("KEYS")
            .arg("pipeline:*")
            .query_async(&mut *conn)
            .await?;

        // Use Redis pipeline for bulk get
        let mut pipe = pipe();
        for key in &keys {
            pipe.get(key);
        }

        let results: Vec<Option<String>> = pipe.query_async(&mut *conn).await?;

        let mut pipelines = Vec::with_capacity(results.len());
        for json_str in results.into_iter().flatten() {
            match serde_json::from_str(&json_str) {
                Ok(pipeline) => pipelines.push(pipeline),
                Err(e) => warn!("Failed to deserialize pipeline: {}", e),
            }
        }

        Ok(pipelines)
    }

    pub async fn get_user_pipelines(
        &self,
        user_id: &str,
    ) -> Result<Vec<Pipeline>, RedisClientError> {
        let mut conn = self.pool.get().await?;

        // Get all keys for the specific user
        let keys: Vec<String> = cmd("KEYS")
            .arg(format!("pipeline:{}:*", user_id))
            .query_async(&mut *conn)
            .await?;

        let mut pipe = pipe();
        for key in &keys {
            pipe.get(key);
        }

        let results: Vec<Option<String>> = pipe.query_async(&mut *conn).await?;

        let mut pipelines = Vec::with_capacity(results.len());
        for json_str in results.into_iter().flatten() {
            match serde_json::from_str(&json_str) {
                Ok(pipeline) => pipelines.push(pipeline),
                Err(e) => warn!("Failed to deserialize pipeline: {}", e),
            }
        }

        Ok(pipelines)
    }

    pub async fn save_all_pipelines(
        &self,
        user_id: &str,
        pipelines: &[Pipeline],
    ) -> Result<(), RedisClientError> {
        let mut conn = self.pool.get().await?;

        for chunk in pipelines.chunks(PIPELINE_BATCH_SIZE) {
            let mut pipe = pipe();

            for pipeline in chunk {
                let key = format!("pipeline:{}:{}", user_id, pipeline.id);
                let value = serde_json::to_string(pipeline)?;
                pipe.set(key, value);
            }

            let _: () = pipe.query_async(&mut *conn).await?;
            debug!("Saved batch of {} pipelines", chunk.len());
        }

        Ok(())
    }

    pub async fn delete_pipeline(&self, user_id: &str, id: &str) -> Result<(), RedisClientError> {
        let mut conn = self.pool.get().await?;
        let _: () = cmd("DEL")
            .arg(format!("pipeline:{}:{}", user_id, id))
            .query_async(&mut *conn)
            .await?;
        Ok(())
    }
}

pub async fn make_redis_client() -> Result<Arc<RedisClient>, RedisClientError> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = RedisClient::new(&redis_url).await?;
    Ok(Arc::new(client))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_redis_client() {
        let client = RedisClient::new("redis://localhost:6379").await.unwrap();

        client
            .set("test_key", &json!({"test": "value"}))
            .await
            .unwrap();

        let value = client.get::<serde_json::Value>("test_key").await.unwrap();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), json!({"test": "value"}));
    }
}
