use crate::completion::generate_completion;
use crate::evolve::EVOLVE_PROMPT;
use crate::memory_note::MemoryNote;
use crate::retriever::{QdrantRetriever, Retriever};
use crate::store::MemoryStore;
use crate::store::MemoryStoreError;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;

pub const K: usize = 5;

pub struct MemorySystem {
    retriever: Arc<QdrantRetriever>,
    store: Arc<MemoryStore>,
}

#[derive(Debug, thiserror::Error)]
pub enum MemorySystemError {
    #[error("Failed to get memory: {0}")]
    GetMemoryError(MemoryStoreError),
}

impl MemorySystem {
    pub async fn from_env() -> Result<Self> {
        let retriever = QdrantRetriever::from_env().await?;
        let store = MemoryStore::from_env().await?;

        Ok(Self {
            retriever: Arc::new(retriever),
            store: Arc::new(store),
        })
    }

    pub async fn update_note_and_embeddings(&self, note: MemoryNote) -> Result<()> {
        // Update in MongoDB
        self.store
            .update_memory(&note.id.to_string(), note.clone())
            .await?;

        // Update in Qdrant
        self.retriever
            .update_document(&note.content, note.to_metadata(), &note.id.to_string())
            .await?;

        Ok(())
    }

    pub async fn persist_note_and_embeddings(&self, note: MemoryNote) -> Result<()> {
        // Persist in MongoDB
        self.store
            .add_memory(&note.id.to_string(), note.clone())
            .await?;

        // Persist in Qdrant
        self.retriever
            .add_document(&note.content, note.to_metadata(), &note.id.to_string())
            .await?;

        Ok(())
    }

    pub async fn add_note(&self, content: String) -> Result<String> {
        // Create a new memory note with LLM analysis
        let note = self
            .process_memory(MemoryNote::with_llm_analysis(content).await?)
            .await?;

        let id = note.id.to_string();

        // Persist the evolved memory
        self.persist_note_and_embeddings(note).await?;

        Ok(id)
    }

    pub async fn get_note(&self, id: &str) -> Result<Option<MemoryNote>, MemorySystemError> {
        self.store
            .get_memory(id)
            .await
            .map_err(MemorySystemError::GetMemoryError)
    }

    pub async fn semantic_search(&self, query: String) -> Result<Vec<MemoryNote>> {
        // Search for related memories using the retriever
        let results = self.retriever.search(&query, K).await?;
        tracing::debug!(
            target: "listen-memory",
            "results: {}",
            serde_json::to_string_pretty(&results).unwrap()
        );

        // Extract document IDs from the results
        let doc_ids = results["ids"]
            .as_array()
            .ok_or_else(|| anyhow!("Failed to get document IDs"))?;

        // Convert to Vec<MemoryNote>
        let mut related_memories = Vec::new();

        for id in doc_ids {
            let id_str = id.as_str().unwrap_or_default();
            let note = self.store.get_memory(id_str).await?;
            if let Some(note) = note {
                related_memories.push(note);
            } else {
                tracing::warn!("Failed to get memory for id: {}", id_str);
            }
        }

        Ok(related_memories)
    }

    pub async fn find_related_memories(&self, query: String, k: usize) -> Result<String> {
        match self.semantic_search(query).await {
            Ok(memories) => {
                if memories.is_empty() {
                    return Ok(String::new());
                }

                let mut memory_str = String::new();
                let mut j = 0;

                // Process main memories
                for memory in memories.iter().take(k) {
                    memory_str.push_str(&format!(
                        "timestamp:{} content:{} context:{} keywords:{:?} memory tags:{:?}\n",
                        memory.timestamp,
                        memory.content,
                        memory.context,
                        memory.keywords,
                        memory.tags
                    ));

                    // Process neighborhood (linked memories)
                    for link in &memory.links {
                        if let Ok(Some(neighbor)) = self.store.get_memory(&link.to_string()).await {
                            memory_str.push_str(&format!(
                            "timestamp:{} memory content:{} memory context:{} memory keywords:{:?} memory tags:{:?}\n",
                            neighbor.timestamp, neighbor.content, neighbor.context, neighbor.keywords, neighbor.tags
                        ));
                            j += 1;
                            if j >= k {
                                break;
                            }
                        }
                    }
                    if j >= k {
                        break;
                    }
                }

                Ok(memory_str)
            }
            Err(e) => {
                tracing::error!(target: "listen-memory", "Error finding related memories: {}", e);
                Ok(String::new())
            }
        }
    }

    pub async fn process_memory(&self, memory: MemoryNote) -> Result<MemoryNote> {
        // Find related memories
        let related_memories = self.semantic_search(memory.content.clone()).await?;

        // Format related memories for the evolution prompt
        let mut nearest_neighbors_text = String::new();
        for (i, mem) in related_memories.iter().enumerate() {
            nearest_neighbors_text.push_str(&format!(
                "memory index:{}\ttalk start time:{}\tmemory content:{}\tmemory context:{}\tmemory keywords:{:?}\tmemory tags:{:?}\n",
                i, mem.timestamp, mem.content, mem.context, mem.keywords, mem.tags
            ));
        }

        // Prepare the evolution prompt
        let prompt = EVOLVE_PROMPT
            .replace("{context}", &memory.context)
            .replace("{content}", &memory.content)
            .replace("{keywords}", &format!("{:?}", memory.keywords))
            .replace("{nearest_neighbors_memories}", &nearest_neighbors_text)
            .replace("{neighbor_number}", &related_memories.len().to_string());

        // Generate completion with the prompt
        let response = generate_completion(&prompt).await?;

        // Parse the response as JSON
        let evolution_response: Value = serde_json::from_str(&response)?;

        let mut evolved_memory = memory.clone();

        // Process the evolution response
        if let Some(true) = evolution_response["should_evolve"].as_bool() {
            if let Some(actions) = evolution_response["actions"].as_array() {
                for action in actions {
                    match action.as_str() {
                        Some("strengthen") => {
                            // Process strengthen action
                            if let Some(connections) =
                                evolution_response["suggested_connections"].as_array()
                            {
                                for conn in connections {
                                    if let Some(conn_idx) = conn.as_u64() {
                                        if conn_idx < related_memories.len() as u64 {
                                            let neighbor = &related_memories[conn_idx as usize];
                                            if !evolved_memory.links.contains(&neighbor.id) {
                                                evolved_memory.links.push(neighbor.id);
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(tags) = evolution_response["tags_to_update"].as_array() {
                                evolved_memory.tags = tags
                                    .iter()
                                    .filter_map(|t| t.as_str().map(String::from))
                                    .collect();
                            }

                            // Add evolution event to history
                            evolved_memory.add_to_evolution_history(format!(
                                "Memory evolved at {} with updated tags: {:?}",
                                Utc::now().to_rfc3339(),
                                evolved_memory.tags
                            ));
                        }
                        Some("update_neighbor") => {
                            // Process update_neighbor action
                            if let (Some(contexts), Some(tags_array)) = (
                                evolution_response["new_context_neighborhood"].as_array(),
                                evolution_response["new_tags_neighborhood"].as_array(),
                            ) {
                                for (i, (context, tags)) in
                                    contexts.iter().zip(tags_array.iter()).enumerate()
                                {
                                    if i < related_memories.len() {
                                        let neighbor_id = related_memories[i].id.to_string();
                                        if let Some(mut neighbor) =
                                            self.store.get_memory(&neighbor_id).await?
                                        {
                                            // Update neighbor context
                                            if let Some(ctx) = context.as_str() {
                                                neighbor.context = ctx.to_string();
                                            }

                                            // Update neighbor tags
                                            if let Some(tag_arr) = tags.as_array() {
                                                neighbor.tags = tag_arr
                                                    .iter()
                                                    .filter_map(|t| t.as_str().map(String::from))
                                                    .collect();
                                            }

                                            self.update_note_and_embeddings(neighbor.clone())
                                                .await?;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(evolved_memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_search() {
        dotenv::dotenv().ok();
        let memory_system = MemorySystem::from_env().await.unwrap();
        let query = "Bitcoin price on 10th of April 2025";
        let results = memory_system
            .semantic_search(query.to_string())
            .await
            .unwrap();
        println!("results: {:?}", results);
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_e2e() {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt::init();
        let memory_system = MemorySystem::from_env().await.unwrap();
        let id = memory_system
            .add_note("Bitcoin has crossed 83k on 10th of April 2025".to_string())
            .await
            .unwrap();
        println!("id: {}", id);
        let note = memory_system.get_note(&id).await.unwrap();
        println!("note: {:#?}", note);
    }
}
