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
    SwapOrder,
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
pub struct WireSwapOrder {
    pub r#type: WireActionType,
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub percentage: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct WireNotification {
    pub r#type: WireActionType,
    pub input_token: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub enum WireAction {
    SwapOrder(WireSwapOrder),
    Notification(WireNotification),
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

        if let Some(first_step) = wire.steps.first() {
            let id = Uuid::new_v4();
            current_steps.push(id);
            steps.insert(id, first_step.into());
        }

        for (i, step) in wire.steps.iter().enumerate().skip(1) {
            let id = Uuid::new_v4();
            let prev_id = steps.iter().nth(i - 1).map(|(id, _)| *id).unwrap();

            if let Some(prev_step) = steps.get_mut(&prev_id) {
                prev_step.next_steps.push(id);
            }

            steps.insert(id, step.into());
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
        }
    }
}

impl From<&WireAction> for Action {
    fn from(wire: &WireAction) -> Self {
        match wire {
            WireAction::SwapOrder(swap) => Action::Order(SwapOrder {
                input_token: swap.input_token.clone(),
                output_token: swap.output_token.clone(),
                amount: swap.amount.clone(),
            }),
            WireAction::Notification(notif) => Action::Notification(Notification {
                message: notif.message.clone(),
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
            WireConditionType::Now => ConditionType::PriceAbove {
                asset: wire.asset.clone(),
                value: 0.0, // Now condition is always true
            },
        };

        Condition {
            condition_type,
            triggered: false,
            last_evaluated: None,
        }
    }
}
