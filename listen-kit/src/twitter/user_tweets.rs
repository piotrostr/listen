use super::{ApiResponse, TwitterApi, TwitterApiError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Types specific to user tweets endpoint
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserTweet {
    #[serde(default)]
    pub r#type: Option<String>,
    pub id: String,
    pub url: Option<String>,
    pub text: String,
    pub source: Option<String>,
    pub retweet_count: Option<u32>,
    pub reply_count: Option<u32>,
    pub like_count: Option<u32>,
    pub quote_count: Option<u32>,
    pub view_count: Option<u32>,
    pub created_at: String,
    pub lang: Option<String>,
    pub bookmark_count: Option<u32>,
    pub is_reply: Option<bool>,
    pub in_reply_to_id: Option<String>,
    pub conversation_id: Option<String>,
    pub in_reply_to_user_id: Option<String>,
    pub in_reply_to_username: Option<String>,
    pub author: Option<super::UserInfo>,
    pub entities: Option<TweetEntities>,
    pub quoted_tweet: Option<Box<UserTweet>>,
    pub retweeted_tweet: Option<Box<UserTweet>>,
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
    pub id_str: String,
    pub name: String,
    pub screen_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserTweetsResponse {
    pub tweets: Vec<UserTweet>,
    #[serde(rename = "pin_tweet")]
    pub pinned_tweet: Option<UserTweet>,
    #[serde(default)]
    pub has_next_page: bool,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FetchUserTweetsOptions {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub include_replies: Option<bool>,
    pub cursor: Option<String>,
}

impl TwitterApi {
    // Fetch user's tweets
    pub async fn fetch_user_tweets(
        &self,
        options: FetchUserTweetsOptions,
    ) -> Result<UserTweetsResponse, TwitterApiError> {
        if options.user_id.is_none() && options.username.is_none() {
            return Err(TwitterApiError::InvalidInput(anyhow::anyhow!(
                "Either user_id or username must be provided"
            )));
        }

        let mut params = HashMap::new();

        if let Some(user_id) = options.user_id {
            params.insert("userId".to_string(), user_id);
        }

        if let Some(username) = options.username {
            params.insert("userName".to_string(), username);
        }

        if let Some(include_replies) = options.include_replies {
            params.insert(
                "includeReplies".to_string(),
                include_replies.to_string(),
            );
        }

        if let Some(cursor) = options.cursor {
            params.insert("cursor".to_string(), cursor);
        }

        let response = self
            .client
            .request::<ApiResponse<UserTweetsResponse>>(
                "/twitter/user/last_tweets",
                Some(params),
            )
            .await?;

        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn twitter_fetch_user_tweets() {
        let twitter = super::TwitterApi::from_env().unwrap();
        let posts = twitter
            .fetch_user_tweets(FetchUserTweetsOptions {
                user_id: None,
                username: Some("listenonsol".to_string()),
                include_replies: Some(false),
                cursor: None,
            })
            .await
            .unwrap();

        println!("{:#?}", posts);
    }
}
