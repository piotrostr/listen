use super::{ApiResponse, TwitterApi, TwitterApiError, TwitterApiProvider};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(alias = "username")]
    pub user_name: Option<String>,
    pub url: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(alias = "verified")]
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
        match self.client.provider() {
            TwitterApiProvider::TwitterApiIo => {
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
            TwitterApiProvider::Xquik => {
                let identifier =
                    super::client::xquik_user_identifier(username)?;
                let endpoint = format!("/x/users/{identifier}");
                self.client.request::<UserInfo>(&endpoint, None).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn twitter_fetch_user_info() {
        let twitter = TwitterApi::from_env().unwrap();
        let user_info = twitter.fetch_user_info("listenonsol").await.unwrap();
        tracing::info!("{:#?}", user_info);
    }

    #[test]
    fn twitter_xquik_user_info_deserialize() {
        let raw_json = r#"{"id":"42","username":"xquikcom","name":"Xquik","description":"X automation API","followers":1200,"following":100,"verified":true,"profilePicture":"https://xquik.com/icon.svg","createdAt":"2026-05-01T00:00:00Z"}"#;

        let user_info = serde_json::from_str::<UserInfo>(raw_json).unwrap();

        assert_eq!(user_info.id.as_deref(), Some("42"));
        assert_eq!(user_info.user_name.as_deref(), Some("xquikcom"));
        assert_eq!(user_info.is_verified, Some(true));
        assert_eq!(user_info.followers, Some(1200));
    }
}
