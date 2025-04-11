use crate::completion::generate_completion;
use anyhow::Result;

const QUERY_PROMPT: &str = r#"
	Generate keywords that best encapsulate the semantic
	meaning of this user prompt.

	Those keywords will be used for finding memories that are most relevant to the
	user prompt and dynamically enriching the assistant context.

	Format your response as a JSON array of strings.

	User prompt:
	{prompt}
	
	Example response format:
	{
		"keywords": [\"keyword1\", \"keyword2\", \"keyword3\"]
	}"#;

pub async fn generate_query(user_prompt: String) -> Result<String> {
    let prompt = QUERY_PROMPT.replace("{prompt}", &user_prompt);
    generate_completion(&prompt).await
}
