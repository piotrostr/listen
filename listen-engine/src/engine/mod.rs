pub mod caip2;
pub mod constants;
pub mod evaluator;
pub mod executor;
pub mod order;
pub mod pipeline;
pub mod privy_config;
pub mod types;
pub mod util;

use crate::engine::evaluator::EvaluatorError;
use crate::redis::client::{make_redis_client, RedisClient, RedisClientError};
use crate::redis::subscriber::{
    make_redis_subscriber, PriceUpdate, RedisSubscriber, RedisSubscriberError,
};
use anyhow::Result;
use metrics::{counter, gauge, histogram};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::evaluator::Evaluator;
use self::pipeline::{Action, Condition, ConditionType, Pipeline, Status};
use crate::server::EngineMessage;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("[Engine] Failed to add pipeline: {0}")]
    AddPipelineError(RedisClientError),

    #[error("[Engine] Failed to delete pipeline: {0}")]
    DeletePipelineError(RedisClientError),

    #[error("[Engine] Failed to get pipeline: {0}")]
    GetPipelineError(String),

    #[error("[Engine] Failed to evaluate pipeline: {0}")]
    EvaluatePipelineError(EvaluatorError),

    #[error("[Engine] Failed to extract assets: {0}")]
    ExtractAssetsError(anyhow::Error),

    #[error("[Engine] Failed to handle price update: {0}")]
    HandlePriceUpdateError(anyhow::Error),

    #[error("[Engine] Executor error: {0}")]
    ExecutorError(executor::ExecutorError),

    #[error("[Engine] Redis client error: {0}")]
    RedisClientError(RedisClientError),

    #[error("[Engine] Redis subscriber error: {0}")]
    RedisSubscriberError(RedisSubscriberError),
}

pub struct Engine {
    pub redis: Arc<RedisClient>,
    pub redis_sub: Arc<RedisSubscriber>,

    receiver: mpsc::Receiver<PriceUpdate>,
    executor: executor::Executor,

    // Active pipelines indexed by UUID
    active_pipelines: RwLock<HashMap<Uuid, Pipeline>>,

    // Asset to pipeline index for efficient updates
    asset_subscriptions: RwLock<HashMap<String, HashSet<Uuid>>>,

    // Current market state
    price_cache: RwLock<HashMap<String, f64>>,
}

impl Engine {
    pub async fn from_env() -> Result<Self, EngineError> {
        let (tx, rx) = mpsc::channel(1000);
        Ok(Self {
            executor: executor::Executor::from_env().map_err(EngineError::ExecutorError)?,
            redis: make_redis_client()
                .await
                .map_err(EngineError::RedisClientError)?,
            redis_sub: make_redis_subscriber(tx).map_err(EngineError::RedisSubscriberError)?,
            receiver: rx,
            active_pipelines: RwLock::new(HashMap::new()),
            asset_subscriptions: RwLock::new(HashMap::new()),
            price_cache: RwLock::new(HashMap::new()),
        })
    }

    pub async fn run(&mut self, mut command_rx: mpsc::Receiver<EngineMessage>) -> Result<()> {
        let pipelines = self.redis.get_all_pipelines().await?;
        let total_pipelines = pipelines.len();
        for pipeline in pipelines {
            self.add_pipeline(pipeline).await?;
        }
        tracing::info!("Added {} pipelines", total_pipelines);

        self.redis_sub.start_listening().await?;

        loop {
            tokio::select! {
                Some(msg) = command_rx.recv() => {
                    match msg {
                        EngineMessage::AddPipeline { pipeline, response_tx } => {
                            let result = self.add_pipeline(pipeline).await;
                            let _ = response_tx.send(result);
                        },
                        EngineMessage::DeletePipeline { user_id, pipeline_id, response_tx } => {
                            let result = self.delete_pipeline(&user_id, pipeline_id).await;
                            let _ = response_tx.send(result);
                        },
                        EngineMessage::GetPipeline { user_id, pipeline_id, response_tx } => {
                            let result = self.get_pipeline(&user_id, pipeline_id).await;
                            let _ = response_tx.send(result);
                        },
                    }
                }
                Some(price_update) = self.receiver.recv() => {
                    if let Err(e) = self.handle_price_update(&price_update.pubkey, price_update.price).await {
                        tracing::error!("Error handling price update: {}", e);
                    }
                }
                else => break,
            }
        }

        Ok(())
    }

    pub async fn add_pipeline(&self, pipeline: Pipeline) -> Result<(), EngineError> {
        if let Err(e) = self.redis.save_pipeline(&pipeline).await {
            return Err(EngineError::AddPipelineError(e));
        }

        // Then add to engine
        let mut active_pipelines = self.active_pipelines.write().await;
        let mut asset_subscriptions = self.asset_subscriptions.write().await;

        // Extract all assets mentioned in pipeline conditions
        let assets = self.extract_assets(&pipeline).await;

        // Update asset subscriptions
        for asset in assets {
            asset_subscriptions
                .entry(asset)
                .or_default()
                .insert(pipeline.id);
        }

        active_pipelines.insert(pipeline.id, pipeline);
        Ok(())
    }

    pub async fn delete_pipeline(
        &self,
        user_id: &str,
        pipeline_id: Uuid,
    ) -> Result<(), EngineError> {
        if let Err(e) = self
            .redis
            .delete_pipeline(&user_id, &pipeline_id.to_string())
            .await
        {
            return Err(EngineError::DeletePipelineError(e));
        }
        let mut active_pipelines = self.active_pipelines.write().await;
        active_pipelines.remove(&pipeline_id);

        Ok(())
    }

    pub async fn get_pipeline(
        &self,
        _user_id: &str,
        pipeline_id: Uuid,
    ) -> Result<Pipeline, EngineError> {
        let active_pipelines = self.active_pipelines.read().await;
        active_pipelines.get(&pipeline_id).cloned().ok_or_else(|| {
            EngineError::GetPipelineError(format!("Pipeline not found: {}", pipeline_id))
        })
    }

    pub async fn handle_price_update(&self, asset: &str, price: f64) -> Result<()> {
        let start = Instant::now();

        counter!("price_updates_processed", 1);

        {
            let mut cache = self.price_cache.write().await;
            cache.insert(asset.to_string(), price);
        }

        // Get affected pipelines
        let subscriptions = self.asset_subscriptions.read().await;
        if let Some(pipeline_ids) = subscriptions.get(asset) {
            for pipeline_id in pipeline_ids {
                if let Some(pipeline) = self.active_pipelines.write().await.get_mut(pipeline_id) {
                    self.evaluate_pipeline(pipeline).await?;
                }
            }
        }

        histogram!("price_update_duration", start.elapsed());

        gauge!(
            "active_pipelines",
            self.active_pipelines.read().await.len() as f64
        );

        Ok(())
    }

    async fn evaluate_pipeline(&self, pipeline: &mut Pipeline) -> Result<(), EngineError> {
        let start = Instant::now();

        let current_step_ids = pipeline.current_steps.clone();
        let price_cache = self.price_cache.read().await.clone();

        for step_id in current_step_ids {
            if let Some(step) = pipeline.steps.get_mut(&step_id) {
                if matches!(step.status, Status::Pending) {
                    match Evaluator::evaluate_conditions(&step.conditions, &price_cache) {
                        Ok(true) => match &step.action {
                            Action::Order(order) => {
                                match self.executor.execute_order(order.clone()).await {
                                    Ok(_) => {
                                        step.status = Status::Completed;
                                        pipeline.current_steps = step.next_steps.clone();
                                        if let Err(e) = self.redis.save_pipeline(&pipeline).await {
                                            return Err(EngineError::AddPipelineError(e));
                                        }
                                    }
                                    Err(e) => {
                                        step.status = Status::Failed;
                                        pipeline.status = Status::Failed;
                                        if let Err(e) = self.redis.save_pipeline(pipeline).await {
                                            return Err(EngineError::AddPipelineError(e));
                                        }
                                        tracing::error!(%step_id, error = %e, "Order execution failed");
                                    }
                                }
                            }
                            Action::Notification(notification) => {
                                tracing::info!(%step_id, ?notification, "TODO: Notification");
                            }
                        },
                        Ok(false) => {
                            // don't do anything
                        }
                        Err(e) => {
                            return Err(EngineError::EvaluatePipelineError(e));
                        }
                    }
                }
            }
        }

        if pipeline.current_steps.is_empty() {
            pipeline.status = Status::Completed;
            if let Err(e) = self.redis.save_pipeline(pipeline).await {
                return Err(EngineError::AddPipelineError(e));
            }
        }

        let duration = start.elapsed();
        counter!("pipeline_evaluations", 1);
        histogram!("pipeline_evaluation_duration", duration);

        Ok(())
    }

    /// Extract all unique assets mentioned in pipeline conditions
    async fn extract_assets(&self, pipeline: &Pipeline) -> HashSet<String> {
        let mut assets = HashSet::new();
        for step in pipeline.steps.values() {
            self.collect_assets_from_condition(&step.conditions, &mut assets)
                .await;
        }
        assets
    }

    async fn collect_assets_from_condition(
        &self,
        conditions: &[Condition],
        assets: &mut HashSet<String>,
    ) {
        let mut stack = Vec::new();
        stack.extend(conditions.iter());

        while let Some(condition) = stack.pop() {
            match &condition.condition_type {
                ConditionType::PriceAbove { asset, .. } => {
                    assets.insert(asset.clone());
                }
                ConditionType::PriceBelow { asset, .. } => {
                    assets.insert(asset.clone());
                }
                ConditionType::PercentageChange { asset, .. } => {
                    assets.insert(asset.clone());
                }
                ConditionType::And(sub_conditions) | ConditionType::Or(sub_conditions) => {
                    stack.extend(sub_conditions.iter());
                }
            }
        }
    }
}
