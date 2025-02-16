use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::order::Order;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    PriceAbove { asset: String, value: f64 },
    PriceBelow { asset: String, value: f64 },
    // PercentageChange {
    //     asset: String,
    //     initial: f64,
    //     value: f64,
    // },
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub triggered: bool,
    pub last_evaluated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Order(Order),
    Notification(Notification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub id: Uuid,
    pub action: Action,
    pub conditions: Vec<Condition>,
    pub next_steps: Vec<Uuid>,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub user_id: String,
    pub current_steps: Vec<Uuid>,
    pub steps: HashMap<Uuid, PipelineStep>,
    pub status: Status,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Pending,   // Not yet started
    Completed, // Successfully finished
    Failed,    // Execution failed
    Cancelled, // Manually cancelled
}
