//! Common logic for evaluating and executing pipeline steps
//! Returns true if the pipeline is done (success or failure/cancelled),
//! false if the pipeline is not complete meaning it should be evaluated
//! again

use std::{collections::HashSet, str::FromStr, time::Instant};

use metrics::{counter, histogram};
use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

use crate::{
    engine::{
        error::EngineError,
        evaluator::Evaluator,
        pipeline::{Action, Pipeline, PipelineStep, Status},
    },
    Engine,
};

impl Engine {
    pub async fn evaluate_pipeline(&self, pipeline: &mut Pipeline) -> Result<bool, EngineError> {
        tracing::debug!("Evaluating pipeline: {}", pipeline.id);
        let start = Instant::now();

        // Get the initial price cache
        let mut price_cache = self.price_cache.read().await.clone();

        // Extract all assets needed for this pipeline
        let needed_assets = self.extract_assets(pipeline).await;

        // Check for missing prices and try to fetch them from Redis
        let missing_assets: Vec<_> = needed_assets
            .iter()
            .filter(|asset| !price_cache.contains_key(*asset) && *asset != "NOW")
            .collect();

        // Validate that all assets are valid Solana pubkeys
        for asset in &needed_assets {
            if *asset != "NOW" && !self.is_valid_solana_asset(asset) {
                // First identify which steps use the invalid asset
                let mut failed_steps = Vec::new();
                let mut steps_to_cancel = Vec::new();

                // Collect steps that use the invalid asset
                for (step_id, step) in &pipeline.steps {
                    if self.step_uses_asset(step, asset).await {
                        failed_steps.push(*step_id);
                        steps_to_cancel.extend(step.next_steps.clone());
                    }
                }

                // Now mark steps as failed and collect downstream steps to cancel
                let mut all_cancelled = Vec::new();
                while !steps_to_cancel.is_empty() {
                    let mut next_to_cancel = Vec::new();

                    for step_id in &steps_to_cancel {
                        if let Some(step) = pipeline.steps.get(step_id) {
                            if !matches!(step.status, Status::Failed | Status::Cancelled) {
                                all_cancelled.push(*step_id);
                                next_to_cancel.extend(step.next_steps.clone());
                            }
                        }
                    }

                    steps_to_cancel = next_to_cancel;
                }

                // Now update all the steps
                for step_id in failed_steps {
                    if let Some(step) = pipeline.steps.get_mut(&step_id) {
                        step.status = Status::Failed;
                        step.error =
                            Some("Only Solana assets with specific mints supported".to_string());
                    }
                }

                for step_id in all_cancelled {
                    if let Some(step) = pipeline.steps.get_mut(&step_id) {
                        step.status = Status::Cancelled;
                    }
                }

                // Mark the pipeline as failed
                pipeline.status = Status::Failed;
                self.redis
                    .save_pipeline(pipeline)
                    .await
                    .map_err(EngineError::SavePipelineError)?;
                return Ok(true);
            }
        }

        if !missing_assets.is_empty() {
            tracing::debug!(
                "Fetching {} missing prices from Redis",
                missing_assets.len()
            );
            for asset in missing_assets {
                if let Some(price) = self.fetch_price_from_redis(asset).await {
                    tracing::debug!("Found price for {} in Redis: {}", asset, price);
                    price_cache.insert(asset.clone(), price);
                }
            }
        }

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
                        let mut cancelled_steps = Vec::new();

                        while let Some(next_step_id) = to_cancel.pop() {
                            // First collect all steps that need to be cancelled
                            if let Some(next_step) = pipeline.steps.get(&next_step_id) {
                                if !matches!(next_step.status, Status::Failed | Status::Cancelled) {
                                    cancelled_steps.push(next_step_id);
                                    to_cancel.extend(next_step.next_steps.clone());
                                }
                            }
                        }

                        // Then update them in a separate loop to avoid multiple mutable borrows
                        for step_id in cancelled_steps {
                            if let Some(next_step) = pipeline.steps.get_mut(&step_id) {
                                next_step.status = Status::Cancelled;
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

    async fn fetch_price_from_redis(&self, asset: &str) -> Option<f64> {
        match self.redis.get_price(asset).await {
            Ok(price) => {
                metrics::counter!("redis_price_fallback_hits", 1);

                // Update the shared in-memory cache for future lookups
                {
                    let mut cache = self.price_cache.write().await;
                    cache.insert(asset.to_string(), price);
                }

                return Some(price);
            }
            _ => {}
        }

        metrics::counter!("redis_price_fallback_misses", 1);
        None
    }

    // Helper method to check if an asset is a valid Solana pubkey
    fn is_valid_solana_asset(&self, asset: &str) -> bool {
        Pubkey::from_str(asset).is_ok()
    }

    // Helper method to check if a step uses a specific asset
    async fn step_uses_asset(&self, step: &PipelineStep, asset: &str) -> bool {
        let mut assets = HashSet::new();
        self.collect_assets_from_condition(&step.conditions, &mut assets)
            .await;
        assets.contains(asset)
    }
}
