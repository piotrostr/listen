use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub status: Status,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterApiResponseError {
    pub error: u32,
    pub message: String,
}

impl std::fmt::Display for TwitterApiResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.error, self.message)
    }
}

#[derive(Debug, Deserialize)]
pub struct TweetsResponse {
    pub tweets: Vec<Tweet>,
    #[serde(rename = "pin_tweet")]
    pub pinned_tweet: Option<Tweet>,
    #[serde(default)]
    pub has_next_page: bool,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

// User Info types
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub user_name: String,
    pub id: String,
    pub name: String,
    pub is_blue_verified: Option<bool>,
    pub profile_picture: Option<String>,
    pub cover_picture: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub followers: Option<u32>,
    pub following: Option<u32>,
    pub can_dm: Option<bool>,
    pub created_at: Option<String>,
    pub statuses_count: Option<u32>,
    // Add other fields as needed
}

// Tweet types
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tweet {
    pub id: String,
    pub url: Option<String>,
    pub text: String,
    pub retweet_count: Option<u32>,
    pub reply_count: Option<u32>,
    pub like_count: Option<u32>,
    pub quote_count: Option<u32>,
    pub created_at: String,
    pub is_reply: Option<bool>,
    pub in_reply_to_id: Option<String>,
    pub author: Option<UserInfo>,
    // Add other fields as needed
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FetchPostsOptions {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub include_replies: Option<bool>,
    pub cursor: Option<String>,
}
