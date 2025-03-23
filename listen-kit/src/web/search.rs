use serde::{Deserialize, Serialize};

use super::{Web, WebError};

impl Web {
    pub async fn search(
        &self,
        query: &str,
    ) -> Result<serde_json::Value, WebError> {
        let body = serde_json::json!({
            "query": query,
        });
        let response = self.client.post("/search", body).await?;
        let mut results: SearchResults = serde_json::from_value(response)?;
        results.results.truncate(6);
        Ok(serde_json::to_value(&results)?)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SearchResults {
    pub results: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let web = Web::from_env().unwrap();
        let response = web.search("tesla").await.unwrap();
        println!("{}", serde_json::to_string_pretty(&response).unwrap());
    }
}
