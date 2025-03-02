use std::hash::{Hash, Hasher};
use std::{collections::HashMap, hash::DefaultHasher};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::order::SwapOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    PriceAbove { asset: String, value: f64 },
    PriceBelow { asset: String, value: f64 },
    Now { asset: String },
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
    Order(SwapOrder),
    Notification(Notification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub id: Uuid,
    pub action: Action,
    pub conditions: Vec<Condition>,
    pub next_steps: Vec<Uuid>,
    pub status: Status,
    pub transaction_hash: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub user_id: String,
    pub wallet_address: String,
    pub pubkey: String,
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

impl Hash for Status {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Status::Pending => state.write_u8(0),
            Status::Completed => state.write_u8(1),
            Status::Failed => state.write_u8(2),
            Status::Cancelled => state.write_u8(3),
        }
    }
}

impl Pipeline {
    pub fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.user_id.hash(&mut hasher);
        self.wallet_address.hash(&mut hasher);
        self.pubkey.hash(&mut hasher);
        self.current_steps.hash(&mut hasher);
        self.steps.len().hash(&mut hasher);
        for id in self.steps.keys() {
            id.hash(&mut hasher);
        }
        self.status.hash(&mut hasher);
        self.created_at.hash(&mut hasher);

        hasher.finish().to_string()
    }
}
