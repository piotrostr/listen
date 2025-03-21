//! Common logic for evaluating and executing pipeline steps
//! Returns true if the pipeline is done (success or failure/cancelled),
//! false if the pipeline is not complete meaning it should be evaluated
//! again

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    time::Instant,
};

use metrics::{counter, histogram};
use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

use crate::{
    engine::{
        error::EngineError,
        evaluator::Evaluator,
        pipeline::{Action, ConditionType, Pipeline, PipelineStep, Status},
    },
    Engine,
};

/// Evaluate methods in Engine return `bool`
/// `true` if the pipeline is complete and should be removed from active pipelines,
/// `false` to continue the pipeline evaluation on price update events
///
/// on error, the pipeline is saved to redis and the error is returned instantly
impl Engine {
    /// ensure that all prices are available for the pipeline,
    pub async fn ensure_prices_available(
        &self,
        pipeline: &mut Pipeline,
    ) -> Result<bool, EngineError> {
        // Get the initial price cache
        let mut price_cache = self.price_cache.read().await.clone();

        // Extract all assets needed for this pipeline
        let needed_assets = self.extract_assets(pipeline);

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
                    if self.step_uses_asset(step, asset) {
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

        Ok(false)
    }

    fn populate_current_steps_if_empty(&self, pipeline: &mut Pipeline) {
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
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn process_all_steps(
        &self,
        pipeline: &mut Pipeline,
        price_cache: &HashMap<String, f64>,
        pipeline_hash: &mut String,
    ) -> Result<(), EngineError> {
        // Collect indexes of steps to remove after processing
        let mut steps_to_remove = Vec::new();
        let mut steps_to_add = Vec::new();

        // Use index-based iteration to allow dropping the mutable borrow
        let mut i = 0;
        while i < pipeline.current_steps.len() {
            let current_step_id = pipeline.current_steps[i];
            let mut step_status_changed = false;

            if let Some(step) = pipeline.steps.get_mut(&current_step_id) {
                match step.status {
                    Status::Completed => {
                        // Step is complete, add its next steps and mark this one for removal
                        steps_to_remove.push(i);
                        if let Some(step) = pipeline.steps.get(&current_step_id) {
                            steps_to_add.extend(step.next_steps.clone());
                        }
                    }
                    Status::Pending => {
                        match Evaluator::evaluate_conditions(&step.conditions, price_cache) {
                            Ok(true) => match &step.action {
                                Action::Order(order) => {
                                    let order = order.clone();
                                    match self
                                        .execute_order(
                                            &order,
                                            &pipeline.user_id,
                                            pipeline.wallet_address.clone(),
                                            pipeline.pubkey.clone(),
                                        )
                                        .await
                                    {
                                        Ok(transaction_hash) => {
                                            step.status = Status::Completed;
                                            step.transaction_hash = Some(transaction_hash);
                                            step_status_changed = true;
                                        }
                                        Err(e) => {
                                            step.status = Status::Failed;
                                            step.transaction_hash = None;
                                            step.error = Some(e.to_string());
                                            step_status_changed = true;

                                            // Only cancel downstream steps if this is not a "Now" condition
                                            if !step.conditions.iter().any(|c| {
                                                matches!(
                                                    c.condition_type,
                                                    ConditionType::Now { .. }
                                                )
                                            }) {
                                                // Cancel all downstream steps
                                                let mut to_cancel = step.next_steps.clone();
                                                let mut cancelled_steps = Vec::new();

                                                while let Some(next_step_id) = to_cancel.pop() {
                                                    if let Some(next_step) =
                                                        pipeline.steps.get(&next_step_id)
                                                    {
                                                        if !matches!(
                                                            next_step.status,
                                                            Status::Failed | Status::Cancelled
                                                        ) {
                                                            cancelled_steps.push(next_step_id);
                                                            to_cancel.extend(
                                                                next_step.next_steps.clone(),
                                                            );
                                                        }
                                                    }
                                                }

                                                // Actually cancel the collected steps
                                                for step_id in cancelled_steps {
                                                    if let Some(next_step) =
                                                        pipeline.steps.get_mut(&step_id)
                                                    {
                                                        next_step.status = Status::Cancelled;
                                                    }
                                                }
                                            }

                                            // Remove this failed step from current_steps
                                            steps_to_remove.push(i);
                                        }
                                    }
                                }
                                Action::Notification(notification) => {
                                    tracing::info!(%current_step_id, ?notification, "Sending notification");
                                    match self
                                        .send_notification(&pipeline.user_id, notification)
                                        .await
                                    {
                                        Ok(res) => {
                                            tracing::info!(
                                                %current_step_id,
                                                ?res,
                                                "Notification sent: {}",
                                                res
                                            );
                                            step.status = Status::Completed;
                                            step_status_changed = true;
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                %current_step_id,
                                                "Failed to send notification: {}",
                                                e
                                            );
                                            step.status = Status::Failed;
                                            step.error = Some(e.to_string());
                                            step_status_changed = true;
                                        }
                                    }
                                }
                            },
                            Ok(false) => {
                                // Conditions not met yet, keep step in current_steps
                            }
                            Err(e) => {
                                // If evaluation fails, mark step as failed but continue with other steps
                                tracing::error!(%current_step_id, error = %e, "Failed to evaluate conditions");
                                step.status = Status::Failed;
                                step.error = Some(e.to_string());
                                step_status_changed = true;

                                // Only cancel downstream steps if this is not a "Now" condition
                                if !step
                                    .conditions
                                    .iter()
                                    .any(|c| matches!(c.condition_type, ConditionType::Now { .. }))
                                {
                                    let mut to_cancel = step.next_steps.clone();
                                    let mut cancelled_steps = Vec::new();

                                    while let Some(next_step_id) = to_cancel.pop() {
                                        if let Some(next_step) = pipeline.steps.get(&next_step_id) {
                                            if !matches!(
                                                next_step.status,
                                                Status::Failed | Status::Cancelled
                                            ) {
                                                cancelled_steps.push(next_step_id);
                                                to_cancel.extend(next_step.next_steps.clone());
                                            }
                                        }
                                    }

                                    for step_id in cancelled_steps {
                                        if let Some(next_step) = pipeline.steps.get_mut(&step_id) {
                                            next_step.status = Status::Cancelled;
                                        }
                                    }
                                }

                                steps_to_remove.push(i);
                            }
                        }
                    }
                    Status::Failed | Status::Cancelled => {
                        // Remove failed or cancelled steps from current_steps
                        steps_to_remove.push(i);
                    }
                }
            } else {
                // Step not found, mark for removal
                steps_to_remove.push(i);
            }

            // Save the pipeline if the step's status changed
            if step_status_changed {
                self.save_pipeline(pipeline, pipeline_hash).await?;
            }

            i += 1;
        }

        // Add new steps to current_steps
        for step_id in steps_to_add {
            if !pipeline.current_steps.contains(&step_id) {
                pipeline.current_steps.push(step_id);
            }
        }

        // Remove steps from current_steps (in reverse order to maintain valid indices)
        steps_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for idx in steps_to_remove {
            if idx < pipeline.current_steps.len() {
                pipeline.current_steps.remove(idx);
            }
        }

        Ok(())
    }

    pub fn collect_step_results(&self, pipeline: &mut Pipeline) -> bool {
        // A pipeline is done when:
        // 1. All steps have a final status (not pending)
        // 2. OR when current_steps is empty and there are no pending steps that could be run

        let all_steps_have_final_status = pipeline
            .steps
            .values()
            .all(|step| !matches!(step.status, Status::Pending));

        let has_pending_steps = pipeline
            .steps
            .values()
            .any(|step| matches!(step.status, Status::Pending));

        // Pipeline is done if all steps have a final status or if there are no current steps
        // and no pending steps that could be activated
        let pipeline_done = all_steps_have_final_status
            || (pipeline.current_steps.is_empty() && !has_pending_steps);

        if pipeline_done {
            // Determine final status based on step results
            let any_step_failed = pipeline
                .steps
                .values()
                .any(|step| matches!(step.status, Status::Failed));

            pipeline.status = if any_step_failed {
                Status::Failed
            } else {
                Status::Completed
            };

            true // Pipeline is complete
        } else {
            false // Pipeline still has steps to process
        }
    }

    /// if evaluate_pipline returns true, means its complete, saved and should be removed from active pipelines
    pub async fn evaluate_pipeline(&self, pipeline: &mut Pipeline) -> Result<bool, EngineError> {
        tracing::debug!("Evaluating pipeline: {}", pipeline.id);
        let start = Instant::now();
        counter!("pipeline_evaluations", 1);

        match self.ensure_prices_available(pipeline).await {
            Ok(true) => {
                self.redis
                    .save_pipeline(pipeline)
                    .await
                    .map_err(EngineError::SavePipelineError)?;
                let duration = start.elapsed();
                histogram!("pipeline_evaluation_duration", duration);
                return Ok(true);
            }
            Err(e) => {
                let duration = start.elapsed();
                histogram!("pipeline_evaluation_duration", duration);
                return Err(e);
            }
            Ok(false) => {} // false means keep going
        }

        // Clone the price cache after ensuring prices are available
        let price_cache = self.price_cache.read().await.clone();

        let mut pipeline_hash = pipeline.hash();

        // Always populate current_steps if empty, not just during processing
        self.populate_current_steps_if_empty(pipeline);

        // If still empty after populating, check if we have any pending steps that aren't in current_steps
        if pipeline.current_steps.is_empty() {
            for (step_id, step) in &pipeline.steps {
                if matches!(step.status, Status::Pending) {
                    pipeline.current_steps.push(*step_id);
                }
            }
        }

        self.save_pipeline(pipeline, &mut pipeline_hash).await?;

        if !pipeline.current_steps.is_empty() {
            self.process_all_steps(pipeline, &price_cache, &mut pipeline_hash)
                .await?;
            self.save_pipeline(pipeline, &mut pipeline_hash).await?;
        }

        let pipeline_done = self.collect_step_results(pipeline);
        self.save_pipeline(pipeline, &mut pipeline_hash).await?;

        let duration = start.elapsed();
        histogram!("pipeline_evaluation_duration", duration);

        Ok(pipeline_done)
    }

    async fn fetch_price_from_redis(&self, asset: &str) -> Option<f64> {
        if let Ok(price) = self.redis.get_price(asset).await {
            metrics::counter!("redis_price_fallback_hits", 1);

            // Update the shared in-memory cache for future lookups
            {
                let mut cache = self.price_cache.write().await;
                cache.insert(asset.to_string(), price);
            }

            return Some(price);
        }

        metrics::counter!("redis_price_fallback_misses", 1);
        None
    }

    // Helper method to check if an asset is a valid Solana pubkey
    fn is_valid_solana_asset(&self, asset: &str) -> bool {
        Pubkey::from_str(asset).is_ok()
    }

    // Helper method to check if a step uses a specific asset
    fn step_uses_asset(&self, step: &PipelineStep, asset: &str) -> bool {
        let mut assets = HashSet::new();
        self.collect_assets_from_condition(&step.conditions, &mut assets);
        assets.contains(asset)
    }

    pub async fn save_pipeline(
        &self,
        pipeline: &Pipeline,
        pipeline_hash: &mut String,
    ) -> Result<(), EngineError> {
        if pipeline.hash() != *pipeline_hash {
            tracing::info!("Saving pipeline: {}", pipeline.id);
            *pipeline_hash = pipeline.hash();
            self.redis
                .save_pipeline(pipeline)
                .await
                .map_err(EngineError::SavePipelineError)
        } else {
            Ok(())
        }
    }
}
