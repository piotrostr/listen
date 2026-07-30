use super::{ApiResponse, TwitterApi, TwitterApiError, TwitterApiProvider};
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
    pub created_at: Option<String>,
    pub lang: Option<String>,
    pub bookmark_count: Option<u32>,
    pub is_reply: Option<bool>,
    pub is_quote_status: Option<bool>,
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
    #[serde(rename = "pin_tweet", alias = "pinned_tweet")]
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

        let FetchUserTweetsOptions {
            user_id,
            username,
            include_replies,
            cursor,
        } = options;

        let mut params = HashMap::new();

        if let Some(include_replies) = include_replies {
            params.insert(
                "includeReplies".to_string(),
                include_replies.to_string(),
            );
        }

        if let Some(cursor) = cursor {
            params.insert("cursor".to_string(), cursor);
        }

        match self.client.provider() {
            TwitterApiProvider::TwitterApiIo => {
                if let Some(user_id) = user_id {
                    params.insert("userId".to_string(), user_id);
                }

                if let Some(username) = username {
                    params.insert("userName".to_string(), username);
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
            TwitterApiProvider::Xquik => {
                let identifier = username
                    .or(user_id)
                    .expect("username or user_id checked above");
                let identifier =
                    super::client::xquik_user_identifier(&identifier)?;

                let endpoint = format!("/x/users/{identifier}/tweets");
                self.client
                    .request::<UserTweetsResponse>(&endpoint, Some(params))
                    .await
            }
        }
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

        tracing::info!("{:#?}", posts);
    }

    #[test]
    fn twitter_xquik_user_tweets_deserialize() {
        let raw_json = r#"{"tweets":[{"id":"1234567890","text":"Xquik timeline item","createdAt":"2026-05-16T12:00:00Z","url":"https://x.com/xquikcom/status/1234567890","likeCount":4,"retweetCount":2,"replyCount":1,"quoteCount":0,"viewCount":100,"bookmarkCount":1,"isReply":false,"isQuoteStatus":false,"author":{"id":"42","username":"xquikcom","name":"Xquik","verified":true}}],"has_next_page":true,"next_cursor":"cursor-1"}"#;

        let response =
            serde_json::from_str::<UserTweetsResponse>(raw_json).unwrap();
        let tweet = response.tweets.first().unwrap();

        assert_eq!(tweet.id, "1234567890");
        assert_eq!(tweet.created_at.as_deref(), Some("2026-05-16T12:00:00Z"));
        assert_eq!(tweet.like_count, Some(4));
        assert_eq!(tweet.is_quote_status, Some(false));
        assert_eq!(
            tweet
                .author
                .as_ref()
                .and_then(|user| user.user_name.as_deref()),
            Some("xquikcom")
        );
        assert!(response.has_next_page);
        assert_eq!(response.next_cursor.as_deref(), Some("cursor-1"));
    }

    #[test]
    fn twitter_xquik_user_tweets_allow_missing_created_at() {
        let raw_json = r#"{"tweets":[{"id":"1234567890","text":"No timestamp"}],"has_next_page":false,"next_cursor":""}"#;

        let response =
            serde_json::from_str::<UserTweetsResponse>(raw_json).unwrap();

        assert_eq!(response.tweets.len(), 1);
        assert_eq!(response.tweets[0].created_at, None);
    }
}
