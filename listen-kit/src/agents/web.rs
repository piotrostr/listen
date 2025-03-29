use crate::{
    agents::image::ViewImage,
    common::{gemini_agent_builder, GeminiAgent},
};
use anyhow::Result;

pub async fn create_web_agent() -> Result<GeminiAgent> {
    let agent = gemini_agent_builder().tool(ViewImage).build();
    Ok(agent)
}
