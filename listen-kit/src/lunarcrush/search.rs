use super::{ApiResponse, LunarCrushApiClient, LunarCrushApiError};
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
    ) -> Result<ApiResponse<Vec<SearchResult>>, LunarCrushApiError> {
        let mut params = std::collections::HashMap::new();
        params.insert("q".to_string(), query.to_string());

        let endpoint = "/search/topics/v1";

        self.request::<ApiResponse<Vec<SearchResult>>>(
            &endpoint,
            Some(params),
        )
        .await
    }
}

#[cfg(test)]
pub fn mock_search_response(query: &str) -> ApiResponse<Vec<SearchResult>> {
    use super::ApiConfig;

    ApiResponse {
        data: vec![SearchResult {
            topic: query.to_string(),
            title: query.to_string().to_uppercase(),
            interaction_score: Some(1000),
        }],
        config: Some(ApiConfig {
            topic: query.to_string(),
            type_: Some("search".to_string()),
            id: "search_id".to_string(),
            generated: chrono::Utc::now().timestamp() as u64,
        }),
    }
}
