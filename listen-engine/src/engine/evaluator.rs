use super::pipeline::{Condition, ConditionType};
use crate::engine::EngineError;
use std::collections::HashMap;

pub struct Evaluator;

#[derive(Debug, thiserror::Error)]
pub enum EvaluatorError {
    #[error("[Evaluator] Failed to evaluate conditions: {0}")]
    EvaluateConditionsError(String),

    #[error("[Evaluator] Failed to evaluate price condition: {0}")]
    PriceEvaluationError(String),

    #[error("[Evaluator] Missing price data for asset: {0}")]
    MissingPriceData(String),

    #[error("[Evaluator] Invalid condition type: {0}")]
    InvalidConditionType(String),
}

impl From<EvaluatorError> for EngineError {
    fn from(err: EvaluatorError) -> Self {
        EngineError::EvaluatePipelineError(err)
    }
}

impl Evaluator {
    pub fn evaluate_conditions(
        conditions: &[Condition],
        prices: &HashMap<String, f64>,
    ) -> Result<bool, EvaluatorError> {
        conditions.iter().try_fold(true, |acc, c| {
            Ok(acc && Self::evaluate_condition(c, prices)?)
        })
    }

    fn evaluate_condition(
        condition: &Condition,
        prices: &HashMap<String, f64>,
    ) -> Result<bool, EvaluatorError> {
        match &condition.condition_type {
            ConditionType::PriceAbove { asset, value } => {
                let price = prices
                    .get(asset)
                    .ok_or_else(|| EvaluatorError::MissingPriceData(asset.clone()))?;
                Ok(price >= value)
            }
            ConditionType::PriceBelow { asset, value } => {
                let price = prices
                    .get(asset)
                    .ok_or_else(|| EvaluatorError::MissingPriceData(asset.clone()))?;
                Ok(price <= value)
            }
            ConditionType::And(sub) => sub.iter().try_fold(true, |acc, c| {
                Ok(acc && Self::evaluate_condition(c, prices)?)
            }),
            ConditionType::Or(sub) => sub.iter().try_fold(false, |acc, c| {
                Ok(acc || Self::evaluate_condition(c, prices)?)
            }),
            ConditionType::Now { .. } => Ok(true),
        }
    }
}
