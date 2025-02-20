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
use dashmap::DashMap;
use metrics::{counter, histogram};
use moka::future::Cache;
use privy::config::PrivyConfig;
use privy::Privy;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
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

    // Current market state
    price_cache: Arc<RwLock<HashMap<String, f64>>>,
    processing_pipelines: Arc<Mutex<HashSet<String>>>,
    active_pipelines: Arc<DashMap<String, HashSet<String>>>, // asset -> pipeline ids
    pipeline_cache: Cache<String, Pipeline>,
}

impl Clone for Engine {
    fn clone(&self) -> Self {
        Self {
            redis: self.redis.clone(),
            redis_sub: self.redis_sub.clone(),
            privy: self.privy.clone(),
            price_cache: self.price_cache.clone(),
            processing_pipelines: self.processing_pipelines.clone(),
            active_pipelines: self.active_pipelines.clone(),
            pipeline_cache: self.pipeline_cache.clone(),
        }
    }
}

impl Engine {
    pub async fn from_env() -> Result<(Self, mpsc::Receiver<PriceUpdate>), EngineError> {
        let (tx, rx) = mpsc::channel(1000);
        let pipeline_cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(30))
            .build();

        Ok((
            Self {
                privy: Arc::new(Privy::new(
                    PrivyConfig::from_env().map_err(EngineError::PrivyConfigError)?,
                )),
                redis: make_redis_client()
                    .await
                    .map_err(EngineError::RedisClientError)?,
                redis_sub: make_redis_subscriber(tx).map_err(EngineError::RedisSubscriberError)?,
                price_cache: Arc::new(RwLock::new(HashMap::new())),
                processing_pipelines: Arc::new(Mutex::new(HashSet::new())),
                active_pipelines: Arc::new(DashMap::new()),
                pipeline_cache,
            },
            rx,
        ))
    }

    pub async fn run(
        engine: Arc<Self>,
        mut receiver: mpsc::Receiver<PriceUpdate>,
        mut command_rx: mpsc::Receiver<EngineMessage>,
    ) -> Result<()> {
        tracing::info!("Engine starting up");

        match engine.redis.get_all_pipelines().await {
            Ok(p) => {
                tracing::info!("{} pipelines from Redis", p.len());
                p
            }
            Err(e) => {
                tracing::error!("Failed to load pipelines from Redis: {}", e);
                return Err(e.into());
            }
        };

        engine.redis_sub.start_listening().await?;

        loop {
            tokio::select! {
                Some(msg) = command_rx.recv() => {
                    tracing::debug!("Received engine message: {:?}", msg);
                    match msg {
                        EngineMessage::AddPipeline { pipeline, response_tx } => {
                            let asset_ids = engine.extract_assets(&pipeline).await;
                            for asset_id in asset_ids {
                                engine.active_pipelines.entry(asset_id.clone()).or_insert_with(HashSet::new).insert(format!("{}:{}", pipeline.user_id, pipeline.id));
                                engine.redis.save_pipeline(&pipeline).await?;
                            }
                            let _ = response_tx.send(Ok(()));
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
                            let result = engine.get_all_pipelines_by_user(&user_id).await;
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
                Some(price_update) = receiver.recv() => {
                    if let Err(e) = engine.handle_price_update(&price_update.pubkey, price_update.price, price_update.slot).await {
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

        let mut save_needed = false;

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

            save_needed = true;
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

        // Only save if changes were made
        if save_needed {
            self.redis
                .save_pipeline(pipeline)
                .await
                .map_err(EngineError::SavePipelineError)?;
        }

        Ok(false)
    }

    pub async fn handle_price_update(&self, asset: &str, price: f64, slot: u64) -> Result<()> {
        let start = Instant::now();
        counter!("price_updates_processed", 1);

        // Update price cache
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(asset.to_string(), price);
        }

        let asset = asset.to_string();

        // Add debug logging
        if let Some(active_pipelines) = self.active_pipelines.get(&asset) {
            tracing::info!(
                "Processing {} pipelines for asset {}",
                active_pipelines.len(),
                asset
            );
            // Get affected pipelines and process in batches
            let pipeline_ids: Vec<String> = active_pipelines.iter().cloned().collect();

            // Process in chunks to limit concurrent Redis connections
            for chunk in pipeline_ids.chunks(10) {
                let mut futures = Vec::new();
                let asset = asset.clone(); // Clone for each chunk

                // First, batch fetch pipelines from Redis
                let mut pipe = bb8_redis::redis::pipe();
                for id in chunk {
                    pipe.get(format!("pipeline:{}", id));
                }

                let pipelines: Vec<Option<Pipeline>> = self.redis.execute_redis_pipe(pipe).await?;

                tracing::info!("Fetched {} pipelines", pipelines.len());

                // Now process the fetched pipelines concurrently
                for (pipeline_id, maybe_pipeline) in chunk.iter().zip(pipelines) {
                    if let Some(mut pipeline) = maybe_pipeline {
                        let self_clone = self.clone();
                        let pipeline_id = pipeline_id.clone();
                        let asset = asset.clone();
                        futures.push(tokio::spawn(async move {
                            // Try to acquire lock for this pipeline
                            let mut processing = self_clone.processing_pipelines.lock().await;
                            if processing.contains(&pipeline_id) {
                                return Ok(());
                            }
                            processing.insert(pipeline_id.clone());
                            drop(processing);

                            let result = async {
                                let is_complete =
                                    self_clone.evaluate_pipeline(&mut pipeline).await?;
                                if is_complete {
                                    self_clone.active_pipelines.entry(asset).and_modify(
                                        |pipelines| {
                                            pipelines.remove(&pipeline_id);
                                        },
                                    );
                                }
                                Ok::<_, EngineError>(())
                            }
                            .await;

                            // Remove pipeline from processing set
                            let mut processing = self_clone.processing_pipelines.lock().await;
                            processing.remove(&pipeline_id);
                            drop(processing);

                            result
                        }));
                    }
                }

                // Wait for this batch to complete
                for future in futures {
                    if let Err(e) = future.await? {
                        tracing::error!("Error processing pipeline: {}", e);
                    }
                }
            }
        } else {
            tracing::debug!("No active pipelines for asset {}", asset);
        }

        histogram!("price_update_duration", start.elapsed());
        println!("{}: {} {} took {:?}", asset, price, slot, start.elapsed());
        Ok(())
    }
}
