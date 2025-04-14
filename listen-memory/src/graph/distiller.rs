use crate::completion::generate_completion;
use anyhow::Result;

const DISTILL_PROMPT: &str = "distill the key information from the following
JSON: {response}. Structure your response as a JSON object with the format of
title, containing any unique identifiers, and a series of bullet points.

Focus on the key information, keep it brief and concise. Factuality and passing
on any unique identifiers is of utmost importance.
";

pub async fn distill(text: &str) -> Result<String> {
    let prompt = DISTILL_PROMPT.replace("{response}", text);
    let res = generate_completion(&prompt).await?;
    Ok(res)
}
