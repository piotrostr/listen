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

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum WireAction {
    #[serde(rename = "SwapOrder")]
    SwapOrder {
        input_token: String,
        output_token: String,
        amount: String,
        #[serde(default)]
        from_chain_caip2: Option<String>,
        #[serde(default)]
        to_chain_caip2: Option<String>,
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
    #[serde(default)]
    pub conditions: Vec<WireCondition>,
}

#[derive(Debug, Deserialize)]
pub struct WirePipeline {
    pub steps: Vec<WireStep>,
}

pub struct PipelineParams {
    pub user_id: String,
    pub wallet_address: Option<String>,
    pub pubkey: Option<String>,
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
            current_steps.push(step_ids[step_ids.len() - 1]);
        } else {
            for &idx in &now_step_indices {
                current_steps.push(step_ids[idx]);
            }
        }

        for (i, (step, id)) in wire.steps.iter().zip(step_ids.iter()).enumerate() {
            let mut pipeline_step: PipelineStep = step.into();

            if !now_step_indices.contains(&i) && i > 0 {
                pipeline_step.next_steps.push(step_ids[i - 1]);
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
        let conditions = if wire.conditions.is_empty() {
            vec![Condition {
                condition_type: ConditionType::Now {
                    asset: String::new(),
                },
                triggered: false,
                last_evaluated: None,
            }]
        } else {
            wire.conditions.iter().map(Into::into).collect()
        };

        PipelineStep {
            id: Uuid::new_v4(),
            action: (&wire.action).into(),
            conditions,
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
            } => {
                const DEFAULT_SOLANA_CHAIN: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";

                Action::Order(SwapOrder {
                    input_token: input_token.clone(),
                    output_token: output_token.clone(),
                    amount: amount.clone(),
                    from_chain_caip2: from_chain_caip2
                        .clone()
                        .unwrap_or_else(|| DEFAULT_SOLANA_CHAIN.to_string()),
                    to_chain_caip2: to_chain_caip2
                        .clone()
                        .unwrap_or_else(|| DEFAULT_SOLANA_CHAIN.to_string()),
                })
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_swap_order_deserialize_with_default_chains() {
        let json = json!({
            "type": "SwapOrder",
            "input_token": "SOL",
            "output_token": "USDC",
            "amount": "1.0"
        });

        let wire_action: WireAction = serde_json::from_value(json).unwrap();

        match &wire_action {
            WireAction::SwapOrder {
                input_token,
                output_token,
                amount,
                from_chain_caip2,
                to_chain_caip2,
            } => {
                assert_eq!(input_token, "SOL");
                assert_eq!(output_token, "USDC");
                assert_eq!(amount, "1.0");
                assert_eq!(from_chain_caip2, &None);
                assert_eq!(to_chain_caip2, &None);

                // Test conversion to Action
                let action: Action = (&wire_action).into();
                if let Action::Order(order) = action {
                    assert_eq!(
                        order.from_chain_caip2,
                        "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
                    );
                    assert_eq!(
                        order.to_chain_caip2,
                        "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
                    );
                } else {
                    panic!("Expected SwapOrder action");
                }
            }
            _ => panic!("Expected SwapOrder variant"),
        }
    }

    #[test]
    fn test_swap_order_deserialize_with_custom_chains() {
        let json = json!({
            "type": "SwapOrder",
            "input_token": "SOL",
            "output_token": "USDC",
            "amount": "1.0",
            "from_chain_caip2": "eip155:1",
            "to_chain_caip2": "eip155:1337"
        });

        let wire_action: WireAction = serde_json::from_value(json).unwrap();
        let action: Action = (&wire_action).into();

        if let Action::Order(order) = action {
            assert_eq!(order.from_chain_caip2, "eip155:1");
            assert_eq!(order.to_chain_caip2, "eip155:1337");
        } else {
            panic!("Expected SwapOrder action");
        }
    }

    #[test]
    fn test_wire_step_with_no_conditions() {
        let json = json!({
            "action": {
                "type": "SwapOrder",
                "input_token": "SOL",
                "output_token": "USDC",
                "amount": "1.0"
            }
        });

        let wire_step: WireStep = serde_json::from_value(json).unwrap();
        assert!(wire_step.conditions.is_empty());

        let pipeline_step: PipelineStep = (&wire_step).into();
        assert_eq!(pipeline_step.conditions.len(), 1);

        match &pipeline_step.conditions[0].condition_type {
            ConditionType::Now { asset } => {
                assert_eq!(asset, "");
            }
            _ => panic!("Expected Now condition type"),
        }
    }
}
