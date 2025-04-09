use crate::completion::generate_completion;
use crate::embed::generate_embedding;
use crate::evolve::PROMPT;
use crate::memory_note::MemoryNote;
use crate::retriever::{QdrantRetriever, Retriever};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub const K: usize = 5;

pub struct MemorySystem {
    retriever: Arc<QdrantRetriever>,
    memories: Arc<Mutex<HashMap<String, MemoryNote>>>,
}

impl MemorySystem {
    pub async fn from_env() -> Result<Self> {
        let retriever = QdrantRetriever::new("memories").await?;

        Ok(Self {
            retriever: Arc::new(retriever),
            memories: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn add_note(&self, content: String) -> Result<String> {
        // Create a new memory note
        let note = MemoryNote::new(content);
        let note_id = note.id.to_string();

        // Store the document in the retriever with metadata
        let metadata = note.to_metadata();
        self.retriever
            .add_document(&note.content, metadata.clone(), &note_id)
            .await?;

        // Store the note in our local memory
        {
            let mut memories = self.memories.lock().unwrap();
            memories.insert(note_id.clone(), note.clone());
        }

        // Process the memory for evolution
        self.process_memory(note).await?;

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
        let memories = self.memories.lock().unwrap();

        for id in doc_ids {
            let id_str = id.as_str().unwrap_or_default();
            if let Some(note) = memories.get(id_str) {
                related_memories.push(note.clone());
            }
        }

        Ok(related_memories)
    }

    pub async fn process_memory(&self, memory: MemoryNote) -> Result<()> {
        // Find related memories
        let related_memories = self.find_related_memories(memory.content.clone()).await?;

        // Format related memories for the evolution prompt
        let mut nearest_neighbors_text = String::new();
        for (i, mem) in related_memories.iter().enumerate() {
            nearest_neighbors_text.push_str(&format!(
                "memory index: {}\tcontent: {}\tcontext: {}\tkeywords: {:?}\ttags: {:?}\n",
                i, mem.content, mem.context, mem.keywords, mem.tags
            ));
        }

        // Prepare the evolution prompt
        let prompt = PROMPT
            .replace("{context}", &memory.context)
            .replace("{content}", &memory.content)
            .replace("{keywords}", &format!("{:?}", memory.keywords))
            .replace("{nearest_neighbors_memories}", &nearest_neighbors_text)
            .replace("{neighbor_number}", &related_memories.len().to_string());

        // Generate completion with the prompt
        let response = generate_completion(&prompt).await?;

        // Parse the response as JSON
        let evolution_response: Value = serde_json::from_str(&response)?;

        // Process the evolution response
        if let Some(should_evolve) = evolution_response["should_evolve"].as_bool() {
            if should_evolve {
                // Process actions
                if let Some(actions) = evolution_response["actions"].as_array() {
                    // First collect all the updates we need to apply
                    struct MemoryUpdate {
                        id: String,
                        new_context: Option<String>,
                        new_tags: Option<Vec<String>>,
                        links_to_add: Vec<Uuid>,
                    }

                    let mut updates = Vec::new();

                    // Add an update for the current memory
                    let mut current_memory_update = MemoryUpdate {
                        id: memory.id.to_string(),
                        new_context: None,
                        new_tags: None,
                        links_to_add: Vec::new(),
                    };

                    for action in actions {
                        match action.as_str() {
                            Some("strengthen") => {
                                // Process strengthen action - update current memory
                                if let Some(connections) =
                                    evolution_response["suggested_connections"].as_array()
                                {
                                    // Collect links to add
                                    for conn in connections {
                                        if let Some(conn_idx) = conn.as_u64() {
                                            if conn_idx < related_memories.len() as u64 {
                                                let neighbor = &related_memories[conn_idx as usize];
                                                current_memory_update
                                                    .links_to_add
                                                    .push(neighbor.id);
                                            }
                                        }
                                    }
                                }

                                if let Some(tags) = evolution_response["tags_to_update"].as_array()
                                {
                                    // Update tags
                                    current_memory_update.new_tags = Some(
                                        tags.iter()
                                            .filter_map(|t| t.as_str().map(String::from))
                                            .collect(),
                                    );
                                }
                            }
                            Some("update_neighbor") => {
                                // Process update_neighbor action
                                if let (Some(contexts), Some(tags_array)) = (
                                    evolution_response["new_context_neighborhood"].as_array(),
                                    evolution_response["new_tags_neighborhood"].as_array(),
                                ) {
                                    // Create updates for each neighbor
                                    for (i, (context, tags)) in
                                        contexts.iter().zip(tags_array.iter()).enumerate()
                                    {
                                        if i < related_memories.len() {
                                            let neighbor_id = related_memories[i].id.to_string();
                                            let new_context = context.as_str().map(String::from);

                                            let new_tags = if let Some(tag_arr) = tags.as_array() {
                                                Some(
                                                    tag_arr
                                                        .iter()
                                                        .filter_map(|t| {
                                                            t.as_str().map(String::from)
                                                        })
                                                        .collect(),
                                                )
                                            } else {
                                                None
                                            };

                                            updates.push(MemoryUpdate {
                                                id: neighbor_id,
                                                new_context,
                                                new_tags,
                                                links_to_add: Vec::new(),
                                            });
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    // Add the current memory update to the list if it has changes
                    if !current_memory_update.links_to_add.is_empty()
                        || current_memory_update.new_tags.is_some()
                    {
                        updates.push(current_memory_update);
                    }

                    // Now apply all updates in a single mutable borrow
                    let mut memories = self.memories.lock().unwrap();
                    for update in updates {
                        if let Some(mem) = memories.get_mut(&update.id) {
                            // Update context if present
                            if let Some(context) = update.new_context {
                                mem.context = context;
                            }

                            // Update tags if present
                            if let Some(tags) = update.new_tags {
                                mem.tags = tags;
                            }

                            // Add links
                            for link in update.links_to_add {
                                if !mem.links.contains(&link) {
                                    mem.links.push(link);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
