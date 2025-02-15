use std::collections::HashMap;

use super::pipeline::{Condition, ConditionType};
pub struct Evaluator;

impl Evaluator {
    pub fn evaluate_conditions(conditions: &[Condition], prices: &HashMap<String, f64>) -> bool {
        conditions
            .iter()
            .all(|c| Self::evaluate_condition(c, prices))
    }

    fn evaluate_condition(condition: &Condition, prices: &HashMap<String, f64>) -> bool {
        match &condition.condition_type {
            ConditionType::PriceAbove { asset, threshold } => {
                prices.get(asset).map(|p| p >= threshold).unwrap_or(false)
            }
            ConditionType::PriceBelow { asset, threshold } => {
                prices.get(asset).map(|p| p <= threshold).unwrap_or(false)
            }
            ConditionType::And(sub) => sub.iter().all(|c| Self::evaluate_condition(c, prices)),
            ConditionType::Or(sub) => sub.iter().any(|c| Self::evaluate_condition(c, prices)),
            // PercentageChange would require historical data tracking
            _ => false,
        }
    }
}
