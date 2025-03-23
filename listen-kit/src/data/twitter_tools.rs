use crate::common::wrap_unsafe;
use crate::distiller::analyst::Analyst;
use crate::twitter::{search::QueryType, TwitterApi};
use anyhow::{anyhow, Result};
use rig_tool_macro::tool;

#[tool(description = "
Performs an advanced search for tweets, passes the search results to an Twitter Analyst,
and returns the summary of the search results given the intent

Parameters:
- query (string): The search query string (e.g. \"AI\" OR \"Twitter\" from:elonmusk)
- query_type (string): The type of search (Latest or Top)
- locale (string): The language of the output of the research, either \"en\" (English) or \"zh\" (Chinese)
- cursor (string): Optional cursor for pagination
- intent (string): The intent of the analysis, passed on to the Twitter Analyst agent

Core Query Structure:
Terms combine with implicit AND: term1 term2
Explicit OR: term1 OR term2
Phrases: \"exact phrase\"
Exclusion: -term or -\"phrase\"
Key Operator Categories
Content: #hashtag, $cashtag, \"quoted phrase\"
Users: from:user, to:user, @user, filter:verified
3. Time: since:YYYY-MM-DD, until:YYYY-MM-DD, within_time:2d
Media: filter:images, filter:videos, filter:media
Engagement: min_retweets:10, min_faves:5, min_replies:3
Type: filter:replies, filter:nativeretweets, filter:quote
Location: near:city, within:10km
Multiple operators can be combined to narrow results: from:nasa filter:images since:2023-01-01 \"mars rover\"
")]
pub async fn search_tweets(
    query: String,
    query_type: String,
    locale: String,
    intent: Option<String>,
    cursor: Option<String>,
) -> Result<String> {
    let twitter = TwitterApi::from_env()
        .map_err(|_| anyhow!("Failed to create TwitterApi"))?;
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|_| anyhow!("Failed to create Analyst"))?;
    let query_type = match query_type.as_str() {
        "Latest" => QueryType::Latest,
        "Top" => QueryType::Top,
        _ => return Err(anyhow!("Invalid query type: {}", query_type)),
    };
    let response = twitter.search_tweets(&query, query_type, cursor).await?;
    let distilled = wrap_unsafe(move || async move {
        analyst
            .analyze_twitter(
                &query,
                &serde_json::to_value(&response)?,
                intent,
            )
            .await
            .map_err(|e| anyhow!("Failed to distill: {}", e))
    })
    .await?;
    Ok(distilled)
}

#[tool(description = "
Fetch a single X (twitter) post by its ID
This is useful for finding out the context of any token or project quickly.

Parameters:
- id (string): The id of the post

Returns a JSON object with the tweet data. 
")]
pub async fn fetch_x_post(id: String) -> Result<serde_json::Value> {
    let twitter = TwitterApi::from_env()
        .map_err(|_| anyhow!("Failed to create TwitterApi"))?;
    let response = twitter
        .fetch_tweets_by_ids(vec![id])
        .await
        .map_err(|e| anyhow!("Failed to fetch X post: {}", e))?;
    let tweet = response.tweets.first().ok_or(anyhow!("No tweet found"))?;
    let tweet_json = serde_json::to_value(tweet)
        .map_err(|e| anyhow!("Failed to parse tweet: {}", e))?;
    Ok(tweet_json)
}

#[tool(description = "
Delegate the X (Twitter) profile name to your Twitter Analyst agent that will fetch the
context and provide a summary of the profile given the intent.

This method might take around 10-15 seconds to return a response

It might contain other profiles, if those are relevant to the context, you can
re-research those proflies calling this same tool

Parameters:
- username (string): The X username, e.g. @elonmusk
- language (string): The language of the output of the research, either \"en\" (English) or \"zh\" (Chinese)
- intent (string): The intent of the analysis, passed on to the Twitter Analyst agent
")]
pub async fn research_x_profile(
    username: String,
    language: String,
    intent: Option<String>,
) -> Result<String> {
    let twitter = TwitterApi::from_env()
        .map_err(|_| anyhow!("Failed to create TwitterApi"))?;
    let analyst = Analyst::from_env_with_locale(language)
        .map_err(|_| anyhow!("Failed to create Analyst"))?;
    wrap_unsafe(move || async move {
        let profile = twitter
            .research_profile(&username)
            .await
            .map_err(|e| anyhow!("{:#?}", e))?;
        let distilled = analyst
            .analyze_twitter(
                &username,
                &serde_json::to_value(&profile)?,
                intent,
            )
            .await
            .map_err(|e| anyhow!("Failed to distill: {}", e))?;
        Ok(distilled)
    })
    .await
}
