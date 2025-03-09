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

#[derive(Debug, Deserialize, Clone)]
pub struct TweetsResponse<'a> {
    pub tweets: Vec<Tweet<'a>>,
    #[serde(rename = "pin_tweet")]
    pub pinned_tweet: Option<Tweet<'a>>,
    #[serde(default)]
    pub has_next_page: bool,
    #[serde(default)]
    pub next_cursor: Option<&'a str>,
}

// User Info types
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo<'a> {
    pub user_name: &'a str,
    pub id: &'a str,
    pub name: &'a str,
    pub is_verified: Option<bool>,
    pub is_blue_verified: Option<bool>,
    pub profile_picture: Option<&'a str>,
    pub cover_picture: Option<&'a str>,
    pub description: Option<&'a str>,
    pub location: Option<&'a str>,
    pub followers: Option<u32>,
    pub following: Option<u32>,
    pub can_dm: Option<bool>,
    pub created_at: Option<&'a str>,
    pub statuses_count: Option<u32>,
    // Add other fields as needed
}

// Tweet types
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tweet<'a> {
    pub id: &'a str,
    pub url: Option<&'a str>,
    pub text: &'a str,
    pub retweet_count: Option<u32>,
    pub reply_count: Option<u32>,
    pub like_count: Option<u32>,
    pub quote_count: Option<u32>,
    pub created_at: &'a str,
    pub is_reply: Option<bool>,
    pub in_reply_to_id: Option<&'a str>,
    pub author: Option<UserInfo<'a>>,
    // Add other fields as needed
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FetchPostsOptions<'a> {
    pub user_id: Option<&'a str>,
    pub username: Option<&'a str>,
    pub include_replies: Option<bool>,
    pub cursor: Option<&'a str>,
}
