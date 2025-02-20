use crate::engine::pipeline::{Condition, ConditionType};
use crate::engine::{Engine, Pipeline};
use std::collections::HashSet;

impl Engine {
    /// Extract all unique assets mentioned in pipeline conditions
    pub async fn extract_assets(&self, pipeline: &Pipeline) -> Vec<String> {
        let mut assets = HashSet::new();
        for step in pipeline.steps.values() {
            self.collect_assets_from_condition(&step.conditions, &mut assets)
                .await;
        }
        assets.into_iter().collect()
    }

    pub async fn collect_assets_from_condition(
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
                ConditionType::And(sub_conditions) | ConditionType::Or(sub_conditions) => {
                    stack.extend(sub_conditions.iter());
                }
                ConditionType::Now => {
                    assets.insert("So11111111111111111111111111111111111111112".to_string());
                    // use solana mint for "now" actions, socket streams a new update every second
                    // this could be a special type, something to consider later
                }
            }
        }
    }
}
