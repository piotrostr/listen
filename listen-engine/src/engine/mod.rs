pub mod api;
pub mod bridge;
pub mod collect;
pub mod constants;
pub mod evaluator;
pub mod execute;
pub mod order;
pub mod pipeline;

use crate::engine::evaluator::EvaluatorError;
use crate::redis::client::{make_redis_client, RedisClient, RedisClientError};
use crate::redis::subscriber::{
    make_redis_subscriber, PriceUpdate, RedisSubscriber, RedisSubscriberError,
};
use anyhow::Result;
use metrics::{counter, gauge, histogram};
use privy::config::PrivyConfig;
use privy::{Privy, PrivyError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::evaluator::Evaluator;
use self::pipeline::{Action, Pipeline, Status};
use crate::server::EngineMessage;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("[Engine] Failed to add pipeline: {0}")]
    AddPipelineError(RedisClientError),

    #[error("[Engine] Failed to save pipeline: {0}")]
    SavePipelineError(RedisClientError),

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

    #[error("[Engine] Transaction error: {0}")]
    TransactionError(privy::tx::PrivyTransactionError),

    #[error("[Engine] Swap order error: {0}")]
    SwapOrderError(order::SwapOrderError),

    #[error("[Engine] Redis client error: {0}")]
    RedisClientError(RedisClientError),

    #[error("[Engine] Redis subscriber error: {0}")]
    RedisSubscriberError(RedisSubscriberError),

    #[error("[Engine] Privy error: {0}")]
    PrivyError(PrivyError),

    #[error("[Engine] Blockhash cache error: {0}")]
    BlockhashCacheError(blockhash_cache::BlockhashCacheError),

    #[error("[Engine] Inject blockhash error: {0}")]
    InjectBlockhashError(anyhow::Error),
}

pub struct Engine {
    pub redis: Arc<RedisClient>,
    pub redis_sub: Arc<RedisSubscriber>,
    pub privy: Arc<Privy>,

    receiver: mpsc::Receiver<PriceUpdate>,

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
            privy: Arc::new(Privy::new(
                PrivyConfig::from_env()
                    .map_err(|e| EngineError::PrivyError(PrivyError::Config(e)))?,
            )),
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
        tracing::info!("Engine starting up");

        let pipelines = match self.redis.get_all_pipelines().await {
            Ok(p) => {
                tracing::info!("Loaded {} pipelines from Redis", p.len());
                p
            }
            Err(e) => {
                tracing::error!("Failed to load pipelines from Redis: {}", e);
                return Err(e.into());
            }
        };

        let total_pipelines = pipelines.len();
        for pipeline in pipelines {
            self.add_pipeline(pipeline).await?;
        }
        tracing::info!("Added {} pipelines", total_pipelines);

        self.redis_sub.start_listening().await?;

        loop {
            tokio::select! {
                Some(msg) = command_rx.recv() => {
                    tracing::debug!("Received engine message: {:?}", msg);
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
                        EngineMessage::GetAllPipelinesByUser { user_id, response_tx } => {
                            tracing::debug!("Getting pipelines for user {}", user_id);
                            let result = self.get_all_pipelines_by_user(&user_id).await;
                            match &result {
                                Ok(pipelines) => tracing::debug!("Found {} pipelines for user", pipelines.len()),
                                Err(e) => tracing::error!("Error getting pipelines: {}", e),
                            }
                            if response_tx.send(result).is_err() {
                                tracing::error!("Failed to send response - channel closed");
                            }
                        },
                    }
                }
                Some(price_update) = self.receiver.recv() => {
                    if let Err(e) = self.handle_price_update(&price_update.pubkey, price_update.price).await {
                        tracing::error!("Error handling price update: {}", e);
                    }
                }
                else => break
            }
        }

        Ok(())
    }

    /// Common logic for evaluating and executing pipeline steps
    async fn evaluate_pipeline(&self, pipeline: &mut Pipeline) -> Result<(), EngineError> {
        let start = Instant::now();
        let price_cache = self.price_cache.read().await.clone();

        // Keep evaluating steps until no more steps are found
        while !pipeline.current_steps.is_empty() {
            let current_step_ids = pipeline.current_steps.clone();
            let mut next_steps = Vec::new();

            for step_id in current_step_ids {
                if let Some(step) = pipeline.steps.get(&step_id) {
                    if matches!(step.status, Status::Pending) {
                        match Evaluator::evaluate_conditions(&step.conditions, &price_cache) {
                            Ok(true) => match &step.action {
                                Action::Order(order) => {
                                    let order = order.clone();
                                    match self.execute_order(pipeline, step_id, &order).await {
                                        Ok(_) => {
                                            // Next steps are already set in execute_order
                                            // Just need to continue processing them
                                        }
                                        Err(e) => {
                                            tracing::error!(%step_id, error = %e, "Failed to execute order");
                                        }
                                    }
                                }
                                Action::Notification(notification) => {
                                    tracing::info!(%step_id, ?notification, "TODO: Notification");
                                    if let Some(step) = pipeline.steps.get_mut(&step_id) {
                                        step.status = Status::Completed;
                                        next_steps.extend(step.next_steps.clone());
                                    }
                                }
                            },
                            Ok(false) => {
                                // If condition isn't met, keep the step in current_steps
                                next_steps.push(step_id);
                            }
                            Err(e) => {
                                return Err(EngineError::EvaluatePipelineError(e));
                            }
                        }
                    }
                }
            }

            // Update current_steps with any new steps
            pipeline.current_steps = next_steps;

            // Save pipeline state after each iteration
            self.redis
                .save_pipeline(pipeline)
                .await
                .map_err(EngineError::SavePipelineError)?;
        }

        // Only mark as completed if there are no more steps and status isn't Failed
        if pipeline.current_steps.is_empty() && !matches!(pipeline.status, Status::Failed) {
            pipeline.status = Status::Completed;
            self.redis
                .save_pipeline(pipeline)
                .await
                .map_err(EngineError::SavePipelineError)?;
        }

        let duration = start.elapsed();
        counter!("pipeline_evaluations", 1);
        histogram!("pipeline_evaluation_duration", duration);

        Ok(())
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
}
