use crate::memory_system::MemorySystem;
use anyhow::Result;

impl MemorySystem {
    pub fn summarize_relevant_memories(
        &self,
        memories_text: String,
        query: String,
    ) -> Result<String> {
        let _prompt = format!(
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

        todo!()
    }
}
