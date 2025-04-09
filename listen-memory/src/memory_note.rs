use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNote {
    pub id: Uuid,
    pub content: String,

    // Semantic metadata
    pub keywords: Vec<String>,
    pub links: Vec<Uuid>, // Representing links as Vec<String> of IDs for simplicity
    pub context: String,
    pub category: String,
    pub tags: Vec<String>,

    // Temporal information
    pub timestamp: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,

    // Usage and evolution data
    pub retrieval_count: u32,
    pub evolution_history: Vec<String>, // Representing history as Vec<String> for simplicity
}
