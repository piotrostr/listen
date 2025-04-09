use crate::completion::generate_completion;
use crate::evolve::EVOLVE_PROMPT;
use crate::memory_note::MemoryNote;
use crate::retriever::{QdrantRetriever, Retriever};
use crate::store::MemoryStore;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;

pub const K: usize = 5;

pub struct MemorySystem {
    retriever: Arc<QdrantRetriever>,
    store: Arc<MemoryStore>,
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

    pub async fn add_note(&self, content: String) -> Result<String> {
        // Create a new memory note with LLM analysis
        let note = MemoryNote::with_llm_analysis(content).await?;
        let note_id = note.id.to_string();

        // Store the document in the retriever with metadata
        let metadata = note.to_metadata();
        self.retriever
            .add_document(&note.content, metadata.clone(), &note_id)
            .await?;

        // Store in MongoDB
        self.store.add_memory(note.clone()).await?;

        // Process the memory for evolution
        let (should_evolve, evolved_note) = self.process_memory(note).await?;

        // If evolution occurred, update the memory
        if should_evolve {
            // Add evolution event to history
            let mut note_to_update = evolved_note.clone();
            note_to_update.add_to_evolution_history(format!(
                "Memory evolved at {} with updated tags: {:?}",
                Utc::now().to_rfc3339(),
                note_to_update.tags
            ));

            // Update in MongoDB
            self.store
                .update_memory(&note_id, note_to_update.clone())
                .await?;

            // Update in retriever
            let metadata = note_to_update.to_metadata();
            self.retriever
                .update_document(&note_to_update.content, metadata, &note_id)
                .await?;
        }

        Ok(note_id)
    }

    pub async fn find_related_memories(&self, query: String) -> Result<Vec<MemoryNote>> {
        // Search for related memories using the retriever
        let results = self.retriever.search(&query, K).await?;

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
            }
        }

        Ok(related_memories)
    }

    pub async fn find_related_memories_raw(&self, query: String, k: usize) -> Result<String> {
        if let Ok(memories) = self.find_related_memories(query).await {
            if memories.is_empty() {
                return Ok(String::new());
            }

            let mut memory_str = String::new();
            let mut j = 0;

            // Process main memories
            for memory in memories.iter().take(k) {
                memory_str.push_str(&format!(
                    "talk start time:{} memory content:{} memory context:{} memory keywords:{:?} memory tags:{:?}\n",
                    memory.timestamp, memory.content, memory.context, memory.keywords, memory.tags
                ));

                // Process neighborhood (linked memories)
                for link in &memory.links {
                    if let Ok(Some(neighbor)) = self.store.get_memory(&link.to_string()).await {
                        memory_str.push_str(&format!(
                            "talk start time:{} memory content:{} memory context:{} memory keywords:{:?} memory tags:{:?}\n",
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
        } else {
            Ok(String::new())
        }
    }

    pub async fn process_memory(&self, memory: MemoryNote) -> Result<(bool, MemoryNote)> {
        // Find related memories
        let related_memories = self.find_related_memories(memory.content.clone()).await?;

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
        let mut should_evolve = false;

        // Process the evolution response
        if let Some(true) = evolution_response["should_evolve"].as_bool() {
            should_evolve = true;

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

                                            // Update the neighbor in storage and retriever
                                            self.store
                                                .update_memory(&neighbor_id, neighbor.clone())
                                                .await?;

                                            let metadata = neighbor.to_metadata();
                                            self.retriever
                                                .update_document(
                                                    &neighbor.content,
                                                    metadata,
                                                    &neighbor_id,
                                                )
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

        Ok((should_evolve, evolved_memory))
    }
}
