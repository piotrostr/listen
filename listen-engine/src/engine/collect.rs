use crate::engine::pipeline::{Condition, ConditionType};
use crate::engine::{Engine, Pipeline};
use std::collections::HashSet;

impl Engine {
    /// Extract all unique assets mentioned in pipeline conditions
    pub fn extract_assets(&self, pipeline: &Pipeline) -> Vec<String> {
        let mut assets = HashSet::new();
        for step in pipeline.steps.values() {
            self.collect_assets_from_condition(&step.conditions, &mut assets);
        }
        assets.into_iter().collect()
    }

    pub fn collect_assets_from_condition(
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
                ConditionType::Now { .. } => {
                    assets.insert("NOW".to_string());
                }
            }
        }
    }
}
