// TODO it might make sense to add a tool for twitter search
// something that has the advanced filters and queries
// e.g. https://github.com/igorbrigadir/twitter-advanced-search

use super::{TwitterApi, TwitterApiError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResponse {
    pub tweets: Vec<serde_json::Value>,
    pub has_next_page: Option<bool>,
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

        tracing::info!("Found {} tweets", search_results.tweets.len());
        tracing::info!("Has next page: {:?}", search_results.has_next_page);
        tracing::info!("Next cursor: {:?}", search_results.next_cursor);
    }

    #[tokio::test]
    async fn twitter_search_tweets_deserialize() {
        let raw_json = r#"{"tweets":[{"type":"tweet","id":"1907156789050613983","url":"https://x.com/arcdotfun/status/1907156789050613983","twitterUrl":"https://twitter.com/arcdotfun/status/1907156789050613983","text":"RT @soulgra_ph: soulgraph 1.0 is now live–create, call, and chat with ai characters that remember you. \n\n100% uncensored.\n\nhere's what's ne…","source":"","retweetCount":33,"replyCount":13,"likeCount":157,"quoteCount":9,"viewCount":[0],"createdAt":"Tue Apr 01 19:43:06 +0000 2025","lang":"en","bookmarkCount":19,"isReply":false,"inReplyToId":"","conversationId":"1907156789050613983","inReplyToUserId":"","inReplyToUsername":"","isPinned":false,"author":{"type":"user","userName":"arcdotfun","url":"https://x.com/arcdotfun","twitterUrl":"https://twitter.com/arcdotfun","id":"1739708260293160960","name":"arc","isVerified":false,"isBlueVerified":true,"profilePicture":"https://pbs.twimg.com/profile_images/1880462137132240896/YW4ReFSR_normal.jpg","coverPicture":"https://pbs.twimg.com/profile_banners/1739708260293160960/1737172012","description":"we take the red pill then the blue pill\n\nca: 61V8..pump","location":"where im needed","followers":49676,"following":7,"status":"","canDm":false,"canMediaTag":true,"createdAt":"Tue Dec 26 18:02:56 +0000 2023","entities":{"description":{},"url":{"urls":[{"display_url":"arc.fun","expanded_url":"https://www.arc.fun/","indices":[0,23],"url":"https://t.co/biBjav2Cy7"}]}},"fastFollowersCount":0,"favouritesCount":0,"hasCustomTimelines":true,"isTranslator":false,"mediaCount":30,"statusesCount":346,"withheldInCountries":[],"affiliatesHighlightedLabel":{},"possiblySensitive":false,"pinnedTweetIds":["1909625026690924602"],"profile_bio":{"description":"we take the red pill then the blue pill\n\nca: 61V8..pump","entities":{"description":{},"url":{"urls":[{"display_url":"arc.fun","expanded_url":"https://www.arc.fun/","indices":[0,23],"url":"https://t.co/biBjav2Cy7"}]}},"withheld_in_countries":[]},"isAutomated":false,"automatedBy":null},"extendedEntities":{},"card":{},"place":{},"entities":{"user_mentions":[{"id_str":"1868774591750881280","indices":[3,14],"name":"soulgraph","screen_name":"soulgra_ph"}]},"isRetweet":false,"isQuote":false,"isConversationControlled":false,"quoted_tweet":null,"retweeted_tweet":{"type":"tweet","id":"1907149945909706758","url":"https://x.com/soulgra_ph/status/1907149945909706758","twitterUrl":"https://twitter.com/soulgra_ph/status/1907149945909706758","text":"soulgraph 1.0 is now live–create, call, and chat with ai characters that remember you. \n\n100% uncensored.\n\nhere's what's new 👇 \n\n(ft. @aeyakovenko's soul)\n\n🧵/ https://t.co/sQMu5LuaIo","source":"","retweetCount":33,"replyCount":13,"likeCount":157,"quoteCount":9,"viewCount":16335,"createdAt":"Tue Apr 01 19:15:54 +0000 2025","lang":"en","bookmarkCount":19,"isReply":false,"inReplyToId":"","conversationId":"1907149945909706758","inReplyToUserId":"","inReplyToUsername":"","isPinned":false,"author":{"type":"user","userName":"soulgra_ph","url":"https://x.com/soulgra_ph","twitterUrl":"https://twitter.com/soulgra_ph","id":"1868774591750881280","name":"soulgraph","isVerified":false,"isBlueVerified":true,"profilePicture":"https://pbs.twimg.com/profile_images/1878515670163353600/ZyW4WYXZ_normal.jpg","coverPicture":"https://pbs.twimg.com/profile_banners/1868774591750881280/1736707312","description":"Create, call, and chat with ai characters that remember you. \n\n100% uncensored. Zero logs.","location":"solana","followers":12442,"following":20,"status":"","canDm":true,"canMediaTag":true,"createdAt":"Mon Dec 16 21:46:13 +0000 2024","entities":{"description":{},"url":{"urls":[{"display_url":"soulgra.ph","expanded_url":"https://soulgra.ph","indices":[0,23],"url":"https://t.co/Grj5khjtXU"}]}},"fastFollowersCount":0,"favouritesCount":0,"hasCustomTimelines":true,"isTranslator":false,"mediaCount":84,"statusesCount":403,"withheldInCountries":[],"affiliatesHighlightedLabel":{},"possiblySensitive":false,"pinnedTweetIds":["1907149945909706758"],"profile_bio":{"description":"Create, call, and chat with ai characters that remember you. \n\n100% uncensored. Zero logs.","entities":{"description":{},"url":{"urls":[{"display_url":"soulgra.ph","expanded_url":"https://soulgra.ph","indices":[0,23],"url":"https://t.co/Grj5khjtXU"}]}},"withheld_in_countries":[]},"isAutomated":false,"automatedBy":null},"extendedEntities":{"media":[{"additional_media_info":{"monetizable":false},"display_url":"pic.twitter.com/sQMu5LuaIo","expanded_url":"https://twitter.com/soulgra_ph/status/1907149945909706758/video/1","ext_media_availability":{"status":"Available"},"id_str":"1907148165473800192","indices":[159,182],"media_key":"7_1907148165473800192","media_results":{"id":"QXBpTWVkaWFSZXN1bHRzOgwAAwoAARp3jSbpGpAACgACGneOxXNagAYAAA==","result":{"__typename":"ApiMedia","id":"QXBpTWVkaWE6DAADCgABGneNJukakAAKAAIad47Fc1qABgAA","media_key":"7_1907148165473800192"}},"media_url_https":"https://pbs.twimg.com/ext_tw_video_thumb/1907148165473800192/pu/img/YaCNczinMAMusvmv.jpg","original_info":{"focus_rects":[],"height":1080,"width":1676},"sizes":{"large":{"h":1080,"w":1676}},"type":"video","url":"https://t.co/sQMu5LuaIo","video_info":{"aspect_ratio":[419,270],"duration_millis":24900,"variants":[{"content_type":"application/x-mpegURL","url":"https://video.twimg.com/ext_tw_video/1907148165473800192/pu/pl/KnVW_0JMZWZpo9sX.m3u8?tag=14"},{"bitrate":256000,"content_type":"video/mp4","url":"https://video.twimg.com/ext_tw_video/1907148165473800192/pu/vid/avc1/418x270/VHwlqM8S1G-lr3s3.mp4?tag=14"},{"bitrate":832000,"content_type":"video/mp4","url":"https://video.twimg.com/ext_tw_video/1907148165473800192/pu/vid/avc1/558x360/cmxDm2NHVpaisxGs.mp4?tag=14"},{"bitrate":2176000,"content_type":"video/mp4","url":"https://video.twimg.com/ext_tw_video/1907148165473800192/pu/vid/avc1/1116x720/M1w7t0YlSkHUlPyB.mp4?tag=14"},{"bitrate":10368000,"content_type":"video/mp4","url":"https://video.twimg.com/ext_tw_video/1907148165473800192/pu/vid/avc1/1676x1080/HmLt0H7XxCCMJFjV.mp4?tag=14"}]}}]},"card":{},"place":{},"entities":{"user_mentions":[{"id_str":"2327407569","indices":[134,146],"name":"toly 🇺🇸","screen_name":"aeyakovenko"}]},"isRetweet":false,"isQuote":false,"isConversationControlled":false,"quoted_tweet":null,"retweeted_tweet":null}}],"status":"success","msg":"success","code":0}"#;
        serde_json::from_str::<super::SearchResponse>(raw_json).unwrap();
    }
}
