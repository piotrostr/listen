use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::error::EngineError;
use crate::engine::order::SwapOrder;
use crate::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    PriceAbove { asset: String, value: f64 },
    PriceBelow { asset: String, value: f64 },
    Now { asset: String },
    // PercentageChange {
    //     asset: String,
    //     initial: f64,
    //     value: f64,
    // },
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub triggered: bool,
    pub last_evaluated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Order(SwapOrder),
    Notification(Notification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub id: Uuid,
    pub action: Action,
    pub conditions: Vec<Condition>,
    pub next_steps: Vec<Uuid>,
    pub status: Status,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub user_id: String,
    pub wallet_address: String,
    pub pubkey: String,
    pub current_steps: Vec<Uuid>,
    pub steps: HashMap<Uuid, PipelineStep>,
    pub status: Status,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Pending,   // Not yet started
    Completed, // Successfully finished
    Failed,    // Execution failed
    Cancelled, // Manually cancelled
}

impl Engine {
    pub async fn register_pipeline(&self, pipeline: &Pipeline) -> Result<(), EngineError> {
        let asset_ids = self.extract_assets(pipeline).await;
        self.redis
            .register_pipeline(pipeline, &asset_ids)
            .await
            .map_err(EngineError::RedisClientError)
    }

    pub async fn unregister_pipeline(&self, pipeline: &Pipeline) -> Result<(), EngineError> {
        let asset_ids = self.extract_assets(pipeline).await;
        self.redis
            .unregister_pipeline(&pipeline.id.to_string(), &asset_ids)
            .await
            .map_err(EngineError::RedisClientError)
    }

    pub async fn get_subscribed_pipelines(
        &self,
        asset_id: &str,
    ) -> Result<Vec<Pipeline>, EngineError> {
        let pipeline_ids = self
            .redis
            .get_pipeline_subscriptions(asset_id)
            .await
            .map_err(EngineError::RedisClientError)?;

        // TODO replace this with redis pipe
        let mut pipelines = Vec::new();
        for id in pipeline_ids {
            if let Some(pipeline) = self
                .redis
                .get_pipeline_by_id(&id)
                .await
                .map_err(EngineError::RedisClientError)?
            {
                pipelines.push(pipeline);
            }
        }

        Ok(pipelines)
    }
}
