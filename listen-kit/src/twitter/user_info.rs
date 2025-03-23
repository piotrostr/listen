use super::{ApiResponse, TwitterApi, TwitterApiError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(default)]
    pub r#type: Option<String>,
    pub user_name: Option<String>,
    pub url: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub is_verified: Option<bool>,
    pub is_blue_verified: Option<bool>,
    pub profile_picture: Option<String>,
    pub cover_picture: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub followers: Option<u32>,
    pub following: Option<u32>,
    pub can_dm: Option<bool>,
    pub created_at: Option<String>,
    pub fast_followers_count: Option<u32>,
    pub favourites_count: Option<u32>,
    pub has_custom_timelines: Option<bool>,
    pub is_translator: Option<bool>,
    pub media_count: Option<u32>,
    pub statuses_count: Option<u32>,
    pub withheld_in_countries: Option<Vec<String>>,
    pub affiliates_highlighted_label:
        Option<HashMap<String, serde_json::Value>>,
    pub possibly_sensitive: Option<bool>,
    pub pinned_tweet_ids: Option<Vec<String>>,
    pub is_automated: Option<bool>,
    pub automated_by: Option<String>,
    pub unavailable: Option<bool>,
    pub message: Option<String>,
    pub unavailable_reason: Option<String>,
    pub profile_bio: Option<ProfileBio>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileBio {
    pub description: Option<String>,
    pub entities: Option<ProfileBioEntities>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileBioEntities {
    pub description: Option<EntityDescription>,
    pub url: Option<EntityDescription>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EntityDescription {
    pub urls: Option<Vec<UrlEntity>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UrlEntity {
    pub display_url: String,
    pub expanded_url: String,
    pub indices: Vec<u32>,
    pub url: String,
}

impl TwitterApi {
    pub async fn fetch_user_info(
        &self,
        username: &str,
    ) -> Result<UserInfo, TwitterApiError> {
        let mut params = std::collections::HashMap::new();
        params.insert("userName".to_string(), username.to_string());

        let response = self
            .client
            .request::<ApiResponse<UserInfo>>(
                "/twitter/user/info",
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
    async fn twitter_fetch_user_info() {
        let twitter = TwitterApi::from_env().unwrap();
        let user_info = twitter.fetch_user_info("listenonsol").await.unwrap();
        println!("{:#?}", user_info);
    }
}
