use crate::distiller::analyst::Analyst;
use crate::signer::SignerContext;
use crate::web::Web;
use crate::{
    common::spawn_with_signer_and_channel, reasoning_loop::ReasoningLoop,
};
use anyhow::{anyhow, Result};
use rig_tool_macro::tool;

#[tool(description = "
Performs a web search using a search engine

Parameters:
- query (string): The search query string
- intent (string): intent of the analysis, helps guide the distillation process, possible to pass \"\" for no specific intent

Returns a distilled summary of the search results processed by an AI analyst
")]
pub async fn search_web(query: String, intent: String) -> Result<String> {
    let locale = SignerContext::current().await.locale();
    let web = Web::from_env().map_err(|_| anyhow!("Failed to create Web"))?;
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|_| anyhow!("Failed to create Analyst"))?;

    let search_results = web.search(&query).await?;

    spawn_with_signer_and_channel(
        SignerContext::current().await,
        ReasoningLoop::get_current_stream_channel().await,
        move || async move {
            analyst
                .analyze_web(
                    &query,
                    &serde_json::to_string(&search_results)?,
                    Some(intent),
                )
                .await
                .map_err(|e| anyhow!("Failed to distill: {}", e))
        },
    )
    .await
    .await?
}

#[tool(description = "
Analyze the content of a specific web page

Parameters:
- url (string): The URL of the web page to analyze
- intent (string): intent of the analysis, helps guide the distillation process, possible to pass \"\" for no specific intent

This tool fetches the content of the specified URL, passes it to an Web Analyst,
and returns the summary of the content given the intent
")]
pub async fn analyze_page_content(
    url: String,
    intent: String,
) -> Result<String> {
    let locale = SignerContext::current().await.locale();
    let web = Web::from_env().map_err(|_| anyhow!("Failed to create Web"))?;
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|_| anyhow!("Failed to create Analyst"))?;

    let page_content = web.contents(url.clone()).await?;

    spawn_with_signer_and_channel(
        SignerContext::current().await,
        ReasoningLoop::get_current_stream_channel().await,
        move || async move {
            analyst
                .analyze_web(
                    &url,
                    &serde_json::to_string(&page_content)?,
                    Some(intent),
                )
                .await
                .map_err(|e| anyhow!("Failed to distill: {}", e))
        },
    )
    .await
    .await?
}
