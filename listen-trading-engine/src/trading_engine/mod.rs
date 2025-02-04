pub mod caip2;
pub mod constants;
pub mod evaluator;
pub mod executor;
pub mod order;
pub mod pipeline;
pub mod privy_config;
pub mod types;
pub mod util;

use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

use anyhow::Result;
use uuid::Uuid;

use self::evaluator::Evaluator;
use self::pipeline::{Condition, ConditionType, Pipeline, Status};

pub struct TradingEngine {
    executor: executor::Executor,

    // Active pipelines indexed by UUID
    active_pipelines: RwLock<HashMap<Uuid, Pipeline>>,

    // Asset to pipeline index for efficient updates
    asset_subscriptions: RwLock<HashMap<String, HashSet<Uuid>>>,

    // Current market state
    price_cache: RwLock<HashMap<String, f64>>,
}

impl TradingEngine {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            executor: executor::Executor::from_env()?,
            active_pipelines: RwLock::new(HashMap::new()),
            asset_subscriptions: RwLock::new(HashMap::new()),
            price_cache: RwLock::new(HashMap::new()),
        })
    }
    pub async fn add_pipeline(&self, pipeline: Pipeline) -> Result<()> {
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

    pub async fn handle_price_update(&self, asset: &str, price: f64) -> Result<()> {
        // Update price cache
        let mut cache = self.price_cache.write().await;
        cache.insert(asset.to_string(), price);
        drop(cache); // Release lock early

        // Get affected pipelines
        let subscriptions = self.asset_subscriptions.read().await;
        if let Some(pipeline_ids) = subscriptions.get(asset) {
            for pipeline_id in pipeline_ids {
                if let Some(pipeline) = self.active_pipelines.write().await.get_mut(pipeline_id) {
                    self.evaluate_pipeline(pipeline).await?;
                }
            }
        }

        Ok(())
    }

    async fn evaluate_pipeline(&self, pipeline: &mut Pipeline) -> Result<()> {
        let current_step_ids = pipeline.current_steps.clone();
        let price_cache = self.price_cache.read().await.clone();

        for step_id in current_step_ids {
            if let Some(step) = pipeline.steps.get_mut(&step_id) {
                if matches!(step.status, Status::Pending)
                    && Evaluator::evaluate_conditions(&step.conditions, &price_cache)
                {
                    // Execute order and update status
                    match self.executor.execute_order(step.order.clone()).await {
                        Ok(_) => {
                            step.status = Status::Completed;
                            pipeline.current_steps = step.next_steps.clone();
                        }
                        Err(e) => {
                            step.status = Status::Failed;
                            pipeline.status = Status::Failed;
                            tracing::error!(%step_id, error = %e, "Order execution failed");
                        }
                    }
                }
            }
        }
        // Check pipeline completion
        if pipeline.current_steps.is_empty() {
            pipeline.status = Status::Completed;
        }

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
