use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilters {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AddMemoryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SearchFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infer: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryItem {
    pub id: String,
    pub memory: String,
    pub hash: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub score: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

impl MemoryItem {
    pub fn distill(&self) -> serde_json::Value {
        let last_updated = self
            .updated_at
            .clone()
            .unwrap_or(self.created_at.clone().unwrap());
        let mut res = serde_json::json!({
            "memory": self.memory,
            "updated_at": last_updated,
            "score": self.score,
        });

        if let Some(metadata) = &self.metadata {
            res["metadata"] = metadata.clone();
        }

        res
    }
}

impl MemoryItem {
    pub fn stringify(&self) -> serde_json::Value {
        let mut res = serde_json::json!({
            "memory": self.memory,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "score": self.score,
        });

        if let Some(metadata) = &self.metadata {
            res["metadata"] = metadata.clone();
        }

        res
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemoryResult {
    pub results: Vec<MemoryItem>,
    pub graph: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub results: Vec<MemoryItem>,
    // TODO possibly some other stuff?
}

pub struct Mem0 {
    base_url: String,
    client: reqwest::Client,
}

impl Default for Mem0 {
    fn default() -> Self {
        Self::new("http://localhost:9696".to_string())
    }
}

impl Mem0 {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn health(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        Ok(response.status().is_success())
    }

    pub async fn add_memory(
        &self,
        messages: Vec<Message>,
        user_id: String,
    ) -> Result<AddMemoryResult> {
        let response = self
            .client
            .post(format!("{}/memories", self.base_url))
            .json(&json!({
                "messages": messages,
                "config": {
                    "user_id": user_id
                }
            }))
            .send()
            .await?;

        let result = response.json().await?;
        Ok(result)
    }

    #[timed::timed]
    pub async fn search_memories(&self, query: String, user_id: String) -> Result<SearchResult> {
        let response = self
            .client
            .post(format!("{}/memories/search", self.base_url))
            .json(&json!({
                "query": query,
                "filters": {
                    "user_id": user_id
                }
            }))
            .send()
            .await?;

        let raw_response = response.text().await?;

        match serde_json::from_str::<SearchResult>(&raw_response) {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("Error: {}", e);
                tracing::error!("Response: {}", raw_response);
                Err(e.into())
            }
        }
    }

    pub async fn get_memory(&self, id: String) -> Result<Option<MemoryItem>> {
        let response = self
            .client
            .get(format!("{}/memories/{}", self.base_url, id))
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json().await?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
