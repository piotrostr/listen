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
use privy::config::PrivyConfig;
use privy::Privy;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use uuid::Uuid;

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
        }
    }
}

impl Engine {
    pub async fn from_env() -> Result<(Self, mpsc::Receiver<PriceUpdate>), EngineError> {
        let (tx, rx) = mpsc::channel(1000);

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

        let existing_pipelines = match engine.redis.get_all_pipelines().await {
            Ok(p) => {
                tracing::info!("{} pipelines from Redis", p.len());
                p
            }
            Err(e) => {
                tracing::error!("Failed to load pipelines from Redis: {}", e);
                return Err(e.into());
            }
        };

        // load existing pipelines into active pipelines
        for pipeline in existing_pipelines {
            let asset_ids = engine.extract_assets(&pipeline).await;
            for asset_id in asset_ids {
                engine
                    .active_pipelines
                    .entry(asset_id.clone())
                    .or_default()
                    .insert(format!("{}:{}", pipeline.user_id, pipeline.id));
            }
        }

        engine.redis_sub.start_listening().await?;

        loop {
            tokio::select! {
                Some(msg) = command_rx.recv() => {
                    tracing::debug!("Received engine message: {:?}", msg);
                    match msg {
                        EngineMessage::AddPipeline { pipeline, response_tx } => {
                            let asset_ids = engine.extract_assets(&pipeline).await;
                            for asset_id in asset_ids {
                                engine.active_pipelines.entry(asset_id.clone()).or_default().insert(format!("{}:{}", pipeline.user_id, pipeline.id));
                                engine.redis.save_pipeline(&pipeline).await?;
                            }
                            let _ = response_tx.send(Ok(pipeline.id.to_string()));
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

        // If current_steps is empty but there are pending steps, populate it from next_steps
        if pipeline.current_steps.is_empty() {
            // Find all steps that are pending and have no previous steps pointing to them
            let root_steps: HashSet<Uuid> = pipeline.steps.keys().cloned().collect();
            let referenced_steps: HashSet<Uuid> = pipeline
                .steps
                .values()
                .flat_map(|step| step.next_steps.clone())
                .collect();

            let entry_steps: Vec<Uuid> =
                root_steps.difference(&referenced_steps).cloned().collect();

            // Add entry steps that are still pending
            for step_id in entry_steps {
                if let Some(step) = pipeline.steps.get(&step_id) {
                    if matches!(step.status, Status::Pending) {
                        pipeline.current_steps.push(step_id);
                        save_needed = true;
                    }
                }
            }

            // Also add any steps that come after completed steps
            for step in pipeline.steps.values() {
                if matches!(step.status, Status::Completed) {
                    for next_step_id in &step.next_steps {
                        if let Some(next_step) = pipeline.steps.get(next_step_id) {
                            // Only add pending steps if there are no failed/cancelled dependencies
                            let has_failed_dependencies = pipeline
                                .steps
                                .values()
                                .filter(|s| s.next_steps.contains(next_step_id))
                                .any(|s| matches!(s.status, Status::Failed | Status::Cancelled));

                            if matches!(next_step.status, Status::Pending)
                                && !pipeline.current_steps.contains(next_step_id)
                                && !has_failed_dependencies
                            {
                                pipeline.current_steps.push(*next_step_id);
                                save_needed = true;
                            }
                        }
                    }
                }
            }
        }

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
                                            step.error = Some(e.to_string());
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
                        // If any step is failed, mark the pipeline as failed and cancel downstream steps
                        pipeline.status = Status::Failed;
                        // Cancel all downstream steps
                        let mut to_cancel = step.next_steps.clone();
                        while let Some(next_step_id) = to_cancel.pop() {
                            if let Some(next_step) = pipeline.steps.get_mut(&next_step_id) {
                                if !matches!(next_step.status, Status::Failed | Status::Cancelled) {
                                    next_step.status = Status::Cancelled;
                                    to_cancel.extend(next_step.next_steps.clone());
                                }
                            }
                        }

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

        let pipeline_ids = {
            let mut res = Vec::new();
            if let Some(now_pipeline_ids) = self.active_pipelines.get(&"NOW".to_string()) {
                res.extend(now_pipeline_ids.iter().cloned());
            }
            if let Some(active_pipelines) = self.active_pipelines.get(&asset.to_string()) {
                res.extend(active_pipelines.iter().cloned());
            }
            res
        };

        // Update price cache after getting pipeline IDs
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(asset.to_string(), price);
        }

        // Process in chunks to limit concurrent Redis connections
        for chunk in pipeline_ids.chunks(10) {
            let mut futures = Vec::new();
            let asset = asset.to_string();

            // Batch fetch pipelines from Redis
            let mut pipe = bb8_redis::redis::pipe();
            for id in chunk {
                pipe.get(format!("pipeline:{}", id));
            }

            let pipelines: Vec<Option<Pipeline>> = self.redis.execute_redis_pipe(pipe).await?;
            tracing::debug!("Fetched {} from redis", pipelines.len());

            // Process the fetched pipelines concurrently
            for (pipeline_id, maybe_pipeline) in chunk.iter().zip(pipelines) {
                if let Some(mut pipeline) = maybe_pipeline {
                    let self_clone = self.clone();
                    let pipeline_id = pipeline_id.clone();
                    let asset = asset.clone();

                    if !matches!(pipeline.status, Status::Pending) {
                        // skip the non-pending pipelines
                        continue;
                    }

                    // Quick check and acquire of processing lock
                    let can_process = {
                        let mut processing = self_clone.processing_pipelines.lock().await;
                        if processing.contains(&pipeline_id) {
                            tracing::warn!("Pipeline {} already being processed", pipeline_id); // TODO remove warn
                            false
                        } else {
                            processing.insert(pipeline_id.clone());
                            true
                        }
                    };

                    if can_process && !matches!(pipeline.status, Status::Failed | Status::Cancelled)
                    {
                        futures.push(tokio::spawn(async move {
                            let result = async {
                                tracing::info!("Evaluating pipeline: {}", pipeline_id);
                                let is_complete =
                                    self_clone.evaluate_pipeline(&mut pipeline).await?;

                                // Only modify active_pipelines if the pipeline is complete
                                if is_complete {
                                    // Minimize time holding the DashMap lock
                                    if let Some(mut pipelines) =
                                        self_clone.active_pipelines.get_mut(&asset)
                                    {
                                        pipelines.remove(&pipeline_id);
                                    }
                                }
                                Ok::<_, EngineError>(())
                            }
                            .await;

                            // Always release the processing lock
                            let mut processing = self_clone.processing_pipelines.lock().await;
                            processing.remove(&pipeline_id);

                            result
                        }));
                    }
                }
            }

            // Wait for this batch to complete
            for future in futures {
                if let Err(e) = future.await? {
                    tracing::error!("Error processing pipeline: {}", e);
                }
            }
        }

        histogram!("price_update_duration", start.elapsed());
        tracing::debug!("{}: {} {} took {:?}", asset, price, slot, start.elapsed());
        Ok(())
    }
}
