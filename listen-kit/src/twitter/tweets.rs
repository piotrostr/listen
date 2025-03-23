use super::{TwitterApi, TwitterApiError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Types specific to tweet details endpoint
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tweet {
    #[serde(default)]
    pub r#type: Option<String>,
    pub id: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub source: Option<String>,
    pub retweet_count: Option<u32>,
    pub reply_count: Option<u32>,
    pub like_count: Option<u32>,
    pub quote_count: Option<u32>,
    pub view_count: Option<u32>,
    pub created_at: Option<String>,
    pub lang: Option<String>,
    pub bookmark_count: Option<u32>,
    pub is_reply: Option<bool>,
    pub in_reply_to_id: Option<String>,
    pub conversation_id: Option<String>,
    pub in_reply_to_user_id: Option<String>,
    pub in_reply_to_username: Option<String>,
    pub author: Option<super::UserInfo>,
    pub entities: Option<TweetEntities>,
    pub quoted_tweet: Option<Box<Tweet>>,
    pub retweeted_tweet: Option<Box<Tweet>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TweetEntities {
    pub hashtags: Option<Vec<Hashtag>>,
    pub urls: Option<Vec<super::user_info::UrlEntity>>,
    pub user_mentions: Option<Vec<UserMention>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hashtag {
    pub indices: Vec<u32>,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserMention {
    pub id_str: Option<String>,
    pub name: Option<String>,
    pub screen_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TweetsResponse {
    pub tweets: Vec<Tweet>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

impl TwitterApi {
    // Get tweets by IDs
    pub async fn fetch_tweets_by_ids(
        &self,
        tweet_ids: Vec<String>,
    ) -> Result<TweetsResponse, TwitterApiError> {
        if tweet_ids.is_empty() {
            return Err(TwitterApiError::InvalidInput(anyhow::anyhow!(
                "At least one tweet ID must be provided"
            )));
        }

        let mut params = HashMap::new();
        params.insert("tweet_ids".to_string(), tweet_ids.join(","));

        let response = self
            .client
            .request::<TweetsResponse>("/twitter/tweets", Some(params))
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn twitter_fetch_tweets_by_ids() {
        let twitter = super::TwitterApi::from_env().unwrap();
        let tweets = twitter
            .fetch_tweets_by_ids(vec![
                // "1898590596442599556".to_string(),
                // "1898591118196900000".to_string(),
                "1895669466786402519".to_string(),
            ])
            .await
            .unwrap();
        std::fs::write(
            "debug/tweets_by_ids.json",
            serde_json::to_string(&tweets).unwrap(),
        )
        .unwrap();
        println!("{:#?}", tweets);
    }
}
