use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::order::SwapOrder;
use super::pipeline::{
    Action, Condition, ConditionType, Notification, Pipeline, PipelineStep, Status,
};

#[derive(Debug, Deserialize)]
pub enum WireActionType {
    #[serde(rename = "SwapOrder")]
    SwapOrder,
    #[serde(rename = "Notification")]
    Notification,
}

#[derive(Debug, Deserialize)]
pub enum WireConditionType {
    #[serde(rename = "PriceAbove")]
    PriceAbove,
    #[serde(rename = "PriceBelow")]
    PriceBelow,
    #[serde(rename = "Now")]
    Now,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WireAction {
    #[serde(rename = "SwapOrder")]
    SwapOrder {
        input_token: String,
        output_token: String,
        amount: String,
        from_chain_caip2: String,
        to_chain_caip2: String,
    },
    #[serde(rename = "Notification")]
    Notification {
        input_token: String,
        message: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct WireCondition {
    pub r#type: WireConditionType,
    pub asset: String,
    pub value: f64,
}

#[derive(Debug, Deserialize)]
pub struct WireStep {
    pub action: WireAction,
    pub conditions: Vec<WireCondition>,
}

#[derive(Debug, Deserialize)]
pub struct WirePipeline {
    pub steps: Vec<WireStep>,
}

pub struct PipelineParams {
    pub user_id: String,
    pub wallet_address: String,
    pub pubkey: String,
}

impl From<(WirePipeline, PipelineParams)> for Pipeline {
    fn from((wire, params): (WirePipeline, PipelineParams)) -> Self {
        let mut steps: HashMap<Uuid, PipelineStep> = HashMap::new();
        let mut current_steps = Vec::new();

        let step_ids: Vec<Uuid> = wire.steps.iter().map(|_| Uuid::new_v4()).collect();

        let now_step_indices: Vec<usize> = wire
            .steps
            .iter()
            .enumerate()
            .filter(|(_, step)| {
                step.conditions
                    .iter()
                    .any(|c| matches!(c.r#type, WireConditionType::Now))
            })
            .map(|(i, _)| i)
            .collect();

        if now_step_indices.is_empty() && !wire.steps.is_empty() {
            current_steps.push(step_ids[0]);
        } else {
            for &idx in &now_step_indices {
                current_steps.push(step_ids[idx]);
            }
        }

        for (i, (step, id)) in wire.steps.iter().zip(step_ids.iter()).enumerate() {
            let mut pipeline_step: PipelineStep = step.into();

            if !now_step_indices.contains(&i) && i + 1 < step_ids.len() {
                pipeline_step.next_steps.push(step_ids[i + 1]);
            }

            steps.insert(*id, pipeline_step);
        }

        Pipeline {
            id: Uuid::new_v4(),
            user_id: params.user_id,
            wallet_address: params.wallet_address,
            pubkey: params.pubkey,
            current_steps,
            steps,
            status: Status::Pending,
            created_at: Utc::now(),
        }
    }
}

impl From<&WireStep> for PipelineStep {
    fn from(wire: &WireStep) -> Self {
        PipelineStep {
            id: Uuid::new_v4(),
            action: (&wire.action).into(),
            conditions: wire.conditions.iter().map(Into::into).collect(),
            next_steps: Vec::new(),
            status: Status::Pending,
            transaction_hash: None,
            error: None,
        }
    }
}

impl From<&WireAction> for Action {
    fn from(wire: &WireAction) -> Self {
        match wire {
            WireAction::SwapOrder {
                input_token,
                output_token,
                amount,
                from_chain_caip2,
                to_chain_caip2,
            } => Action::Order(SwapOrder {
                input_token: input_token.clone(),
                output_token: output_token.clone(),
                amount: amount.clone(),
                from_chain_caip2: from_chain_caip2.clone(),
                to_chain_caip2: to_chain_caip2.clone(),
            }),
            WireAction::Notification { message, .. } => Action::Notification(Notification {
                message: message.clone(),
            }),
        }
    }
}

impl From<&WireCondition> for Condition {
    fn from(wire: &WireCondition) -> Self {
        let condition_type = match wire.r#type {
            WireConditionType::PriceAbove => ConditionType::PriceAbove {
                asset: wire.asset.clone(),
                value: wire.value,
            },
            WireConditionType::PriceBelow => ConditionType::PriceBelow {
                asset: wire.asset.clone(),
                value: wire.value,
            },
            WireConditionType::Now => ConditionType::Now {
                asset: wire.asset.clone(),
            },
        };

        Condition {
            condition_type,
            triggered: false,
            last_evaluated: None,
        }
    }
}
