use anyhow::Result;
use privy::util::base64encode;
use rig::{
    completion::Prompt,
    message::{ContentFormat, Image, ImageMediaType},
};
use rig_tool_macro::tool;

use crate::common::{gemini_agent_builder, wrap_unsafe};

#[tool(description = "View an image")]
pub async fn view_image(image_url: String) -> Result<String> {
    let agent = gemini_agent_builder().preamble("You are a helpful assistant that can read images and describe them").build();
    let image_data = reqwest::get(image_url).await?.bytes().await?;
    let data = base64encode(&image_data);
    wrap_unsafe(move || async move {
        agent
            .prompt(Image {
                data,
                format: Some(ContentFormat::Base64),
                media_type: Some(ImageMediaType::PNG), // this doesn't matter for Gemini
                detail: None,
            })
            .await
            .map_err(|e| anyhow::anyhow!("Error viewing image: {}", e))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_image() {
        let res = view_image("https://ipfs.io/ipfs/QmX1UG3uu6dzQaEycNnwea9xRSwZbGPFEdv8XPXJjBUVsT".to_string())
            .await
            .unwrap();
        println!("{}", res);
    }
}
