use super::{Web, WebError};

impl Web {
    pub async fn contents(
        &self,
        url: String,
    ) -> Result<serde_json::Value, WebError> {
        let body = serde_json::json!({
            "ids": [url],
        });
        self.client
            .post("/contents", body)
            .await
            .map_err(|e| WebError::ExaClientError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_contents() {
        let web = Web::from_env().unwrap();
        let contents = web
            .contents("https://www.infinitebackrooms.com/dreams/conversation-1721540624-scenario-terminal-of-truths-txt".to_string())
            .await
            .unwrap();
        println!("{}", serde_json::to_string_pretty(&contents).unwrap());
    }
}
