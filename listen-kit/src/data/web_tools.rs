use crate::common::wrap_unsafe;
use crate::distiller::analyst::Analyst;
use crate::web::Web;
use anyhow::{anyhow, Result};
use rig_tool_macro::tool;

#[tool(description = "
Performs a web search using a search engine

Parameters:
- query (string): The search query string
- locale (string): The language of the output of the research, either \"en\" (English) or \"zh\" (Chinese)
- intent (string): Optional intent of the analysis, helps guide the distillation process

Returns a distilled summary of the search results processed by an AI analyst
")]
pub async fn search_web(
    query: String,
    locale: String,
    intent: Option<String>,
) -> Result<String> {
    let web = Web::from_env().map_err(|_| anyhow!("Failed to create Web"))?;
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|_| anyhow!("Failed to create Analyst"))?;

    let search_results = web.search(&query).await?;

    let distilled = wrap_unsafe(move || async move {
        analyst
            .analyze_web(
                &query,
                &serde_json::to_string(&search_results)?,
                intent,
            )
            .await
            .map_err(|e| anyhow!("Failed to distill: {}", e))
    })
    .await?;

    Ok(distilled)
}

#[tool(description = "
Analyze the content of a specific web page

Parameters:
- url (string): The URL of the web page to analyze
- locale (string): The language of the output of the analysis, either \"en\" (English) or \"zh\" (Chinese)
- intent (string): Optional intent of the analysis, helps guide the analysis process

This tool fetches the content of the specified URL, passes it to an Web Analyst,
and returns the summary of the content given the intent
")]
pub async fn analyze_page_content(
    url: String,
    locale: String,
    intent: Option<String>,
) -> Result<String> {
    let web = Web::from_env().map_err(|_| anyhow!("Failed to create Web"))?;
    let analyst = Analyst::from_env_with_locale(locale)
        .map_err(|_| anyhow!("Failed to create Analyst"))?;

    let page_content = web.contents(url.clone()).await?;

    let distilled = wrap_unsafe(move || async move {
        analyst
            .analyze_web(&url, &serde_json::to_string(&page_content)?, intent)
            .await
            .map_err(|e| anyhow!("Failed to distill: {}", e))
    })
    .await?;

    Ok(distilled)
}
