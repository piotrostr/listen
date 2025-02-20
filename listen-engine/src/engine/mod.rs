pub mod api;
pub mod bridge;
pub mod collect;
pub mod constants;
pub mod error;
pub mod evaluator;
pub mod execute;
pub mod order;
pub mod pipeline;

use crate::engine::error::EngineError;
use crate::redis::client::{make_redis_client, RedisClient};
use crate::redis::subscriber::{make_redis_subscriber, PriceUpdate, RedisSubscriber};
use anyhow::Result;
use metrics::{counter, histogram};
use privy::config::PrivyConfig;
use privy::{Privy, PrivyError};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use self::evaluator::Evaluator;
use self::pipeline::{Action, Pipeline, Status};
use crate::server::EngineMessage;

pub struct Engine {
    pub redis: Arc<RedisClient>,
    pub redis_sub: Arc<RedisSubscriber>,
    pub privy: Arc<Privy>,

    receiver: mpsc::Receiver<PriceUpdate>,

    // Current market state
    price_cache: RwLock<HashMap<String, f64>>,
    processing_pipelines: Arc<Mutex<HashSet<String>>>,
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
            price_cache: RwLock::new(HashMap::new()),
            processing_pipelines: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    pub async fn run(&mut self, mut command_rx: mpsc::Receiver<EngineMessage>) -> Result<()> {
        tracing::info!("Engine starting up");

        match self.redis.get_all_pipelines().await {
            Ok(p) => {
                tracing::info!("{} pipelines from Redis", p.len());
                p
            }
            Err(e) => {
                tracing::error!("Failed to load pipelines from Redis: {}", e);
                return Err(e.into());
            }
        };

        self.redis_sub.start_listening().await?;

        loop {
            tokio::select! {
                Some(msg) = command_rx.recv() => {
                    tracing::debug!("Received engine message: {:?}", msg);
                    match msg {
                        EngineMessage::AddPipeline { pipeline, response_tx } => {
                            let result = self.register_pipeline(&pipeline).await;
                            let _ = response_tx.send(result);
                        },
                        EngineMessage::DeletePipeline { .. } => {
                            panic!("DeletePipeline not implemented");
                            // let result = self.delete_pipeline(&user_id, pipeline_id).await;
                            // let _ = response_tx.send(result);
                        },
                        EngineMessage::GetPipeline { .. } => {
                            panic!("GetPipeline not implemented");
                        },
                        EngineMessage::GetAllPipelinesByUser { user_id, response_tx } => {
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
    /// Returns true if the pipeline is done (success or failure/cancelled),
    /// false if the pipeline is not complete meaning it should be evaluated
    /// again
    async fn evaluate_pipeline(&self, pipeline: &mut Pipeline) -> Result<bool, EngineError> {
        let start = Instant::now();
        let price_cache = self.price_cache.read().await.clone();

        // Process one step at a time
        while let Some(&current_step_id) = pipeline.current_steps.first() {
            if let Some(step) = pipeline.steps.get_mut(&current_step_id) {
                match step.status {
                    Status::Completed => {
                        // Step is complete, remove it and add next steps
                        pipeline.current_steps.remove(0);
                        if let Some(step) = pipeline.steps.get(&current_step_id) {
                            pipeline.current_steps.extend(step.next_steps.clone());
                        }
                    }
                    Status::Pending => {
                        match Evaluator::evaluate_conditions(&step.conditions, &price_cache) {
                            Ok(true) => match &step.action {
                                Action::Order(order) => {
                                    let order = order.clone();
                                    match self
                                        .execute_order(
                                            &order,
                                            &pipeline.user_id,
                                            &pipeline.wallet_address,
                                            &pipeline.pubkey,
                                        )
                                        .await
                                    {
                                        Ok(transaction_hash) => {
                                            step.status = Status::Completed;
                                            step.transaction_hash = Some(transaction_hash);
                                            continue;
                                        }
                                        Err(e) => {
                                            tracing::error!(%current_step_id, error = %e, "Failed to execute order");
                                            step.status = Status::Failed;
                                            step.transaction_hash = None;
                                            continue;
                                        }
                                    }
                                }
                                Action::Notification(notification) => {
                                    tracing::info!(%current_step_id, ?notification, "TODO: Notification");
                                    step.status = Status::Completed;
                                    continue;
                                }
                            },
                            Ok(false) => {
                                break; // just pending, we'll check again next time
                            }
                            Err(e) => {
                                // if it went wrong (no pricing etc), save pipeline to redis and return
                                self.redis
                                    .save_pipeline(pipeline)
                                    .await
                                    .map_err(EngineError::RedisClientError)?;
                                return Err(EngineError::EvaluatePipelineError(e));
                            }
                        }
                    }
                    Status::Failed => {
                        // If any step is failed, mark the pipeline as failed
                        pipeline.status = Status::Failed;
                        pipeline.current_steps.clear(); // Clear remaining steps
                        break;
                    }
                    Status::Cancelled => {
                        // If any step is cancelled, mark the pipeline as cancelled
                        pipeline.status = Status::Cancelled;
                        pipeline.current_steps.clear(); // Clear remaining steps
                        break;
                    }
                }
            } else {
                // Step not found, remove it
                pipeline.current_steps.remove(0);
            }

            // Save pipeline state after each step
            self.redis
                .save_pipeline(pipeline)
                .await
                .map_err(EngineError::SavePipelineError)?;
        }

        // Check if all steps in the pipeline are complete
        let all_steps_complete = pipeline
            .steps
            .values()
            .all(|step| matches!(step.status, Status::Completed));

        let any_of_steps_failed = pipeline
            .steps
            .values()
            .any(|step| matches!(step.status, Status::Failed | Status::Cancelled));

        if any_of_steps_failed {
            pipeline.status = Status::Failed;
            self.redis
                .save_pipeline(pipeline)
                .await
                .map_err(EngineError::SavePipelineError)?;
            return Ok(true);
        }

        if all_steps_complete {
            pipeline.status = Status::Completed;
            self.redis
                .save_pipeline(pipeline)
                .await
                .map_err(EngineError::SavePipelineError)?;
            return Ok(true);
        }

        let duration = start.elapsed();
        counter!("pipeline_evaluations", 1);
        histogram!("pipeline_evaluation_duration", duration);

        Ok(false)
    }

    pub async fn handle_price_update(&self, asset: &str, price: f64) -> Result<()> {
        let start = Instant::now();
        counter!("price_updates_processed", 1);

        {
            // TODO price cache can pull from redis too
            let mut cache = self.price_cache.write().await;
            cache.insert(asset.to_string(), price);
        }

        // Get affected pipelines
        let subscriptions = self.redis.get_pipeline_subscriptions(asset).await?;

        for pipeline_id in subscriptions {
            // Try to acquire lock for this pipeline
            let mut processing = self.processing_pipelines.lock().await;
            if processing.contains(&pipeline_id) {
                tracing::debug!(
                    "Pipeline {} is already being processed, skipping",
                    pipeline_id
                );
                continue;
            }
            processing.insert(pipeline_id.clone());
            drop(processing); // Release the lock early

            // Process the pipeline
            if let Some(mut pipeline) = self.redis.get_pipeline_by_id(&pipeline_id).await? {
                let is_complete = self.evaluate_pipeline(&mut pipeline).await?;
                if is_complete {
                    self.unregister_pipeline(&pipeline).await?;
                }
            }

            // Remove pipeline from processing set
            let mut processing = self.processing_pipelines.lock().await;
            processing.remove(&pipeline_id);
            drop(processing);
        }

        histogram!("price_update_duration", start.elapsed());
        Ok(())
    }
}
