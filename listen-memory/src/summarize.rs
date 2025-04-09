use crate::completion::generate_completion;
use crate::memory_system::MemorySystem;
use anyhow::Result;
use serde_json::Value;

impl MemorySystem {
    pub async fn summarize_relevant_memories(
        &self,
        memories_text: String,
        query: String,
    ) -> Result<String> {
        let prompt = format!(
            "Given the following conversation memories and a question, select the
            most relevant parts of the conversation that would help answer the
            query. Include the date/time if available.

            Conversation memories:
            {}

            Query: 
            {}

            Return only the relevant parts of the conversation that would help
            answer this specific question. If no parts are relevant, do not do any
            things just return the input. Format your response as a JSON object
            with a \"relevant_parts\" field containing the selected text.",
            memories_text, query,
        );

        // Generate completion
        let response = generate_completion(&prompt).await?;

        // Parse the response to extract relevant parts
        match serde_json::from_str::<Value>(&response) {
            Ok(json) => {
                if let Some(relevant_parts) = json.get("relevant_parts") {
                    if let Some(text) = relevant_parts.as_str() {
                        return Ok(text.to_string());
                    }
                }
                // If parsing succeeds but the format is unexpected, return the full response
                Ok(response)
            }
            Err(_) => {
                // If parsing fails, just return the raw response
                Ok(response)
            }
        }
    }
}
