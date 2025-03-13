// TODO it might make sense to add a tool for twitter search
// something that has the advanced filters and queries
// e.g. https://github.com/igorbrigadir/twitter-advanced-search

use super::{TwitterApi, TwitterApiError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResponse {
    pub tweets: Vec<super::tweets::Tweet>,
    pub has_next_page: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum QueryType {
    Latest,
    Top,
}

impl QueryType {
    fn as_str(&self) -> &'static str {
        match self {
            QueryType::Latest => "Latest",
            QueryType::Top => "Top",
        }
    }
}

impl TwitterApi {
    // TODO distill this response, this yields a lot of output
    pub async fn search_tweets(
        &self,
        query: &str,
        query_type: QueryType,
        cursor: Option<String>,
    ) -> Result<SearchResponse, TwitterApiError> {
        if query.is_empty() {
            return Err(TwitterApiError::InvalidInput(anyhow::anyhow!(
                "Search query cannot be empty"
            )));
        }

        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params
            .insert("queryType".to_string(), query_type.as_str().to_string());

        if let Some(cursor_val) = cursor {
            params.insert("cursor".to_string(), cursor_val);
        }

        let response = self
            .client
            .request::<SearchResponse>(
                "/twitter/tweet/advanced_search",
                Some(params),
            )
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn twitter_search_tweets() {
        let twitter = super::TwitterApi::from_env().unwrap();
        let search_results = twitter
            .search_tweets("AI from:elonmusk", super::QueryType::Latest, None)
            .await
            .unwrap();

        std::fs::write(
            "debug/search_results.json",
            serde_json::to_string(&search_results).unwrap(),
        )
        .unwrap();

        println!("Found {} tweets", search_results.tweets.len());
        println!("Has next page: {}", search_results.has_next_page);
        println!("Next cursor: {:?}", search_results.next_cursor);
    }
}
