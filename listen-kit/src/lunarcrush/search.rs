use super::{LunarCrushApiClient, LunarCrushApiError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub topic: String,
    pub title: String,
    pub interaction_score: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub count: usize,
}

impl LunarCrushApiClient {
    pub async fn search(
        &self,
        query: &str,
    ) -> Result<serde_json::Value, LunarCrushApiError> {
        let mut params = std::collections::HashMap::new();
        params.insert("q".to_string(), query.to_string());

        let endpoint = "/search/topics/v1";

        self.request::<serde_json::Value>(&endpoint, Some(params))
            .await
    }
}
