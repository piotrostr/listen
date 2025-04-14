pub mod agent;
pub mod client;
pub mod distiller;
pub mod prompts;
pub mod tools;

#[cfg(test)]
mod test_e2e;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use agent::{extract_tool_calls, get_tool_calls};
use anyhow::Result;
use client::{GraphEntity, Neo4jClient};
use prompts::EXTRACT_ENTITIES_PROMPT;
use tools::extract_entities_tool;

use crate::graph::{
    client::{remove_spaces_from_entities, RelationResult},
    prompts::{DELETE_RELATIONS_PROMPT, EXTRACT_RELATIONS_PROMPT},
    tools::{delete_memory_tool_graph, noop_tool, relations_tool},
};

pub struct GraphMemory {
    client: Neo4jClient,
}

impl GraphMemory {
    pub async fn from_env() -> Result<Self> {
        let client = Neo4jClient::from_env().await?;
        Ok(Self { client })
    }
}

#[derive(Debug, Clone)]
pub struct Filters {}

pub struct Entity {
    pub name: String,
    pub entity_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddResult {
    pub deleted_entities: Vec<HashMap<String, String>>,
    pub added_entities: Vec<HashMap<String, String>>,
}

impl GraphMemory {
    pub async fn search(
        &self,
        query: &str,
        _filters: Filters,
        limit: Option<usize>,
    ) -> Result<Vec<RelationResult>> {
        println!("searching for: {}", query);
        let limit = limit.unwrap_or(15);
        let search_output = self
            .client
            .search_graph_db(query.to_string(), None, Some(limit))
            .await?;

        if search_output.is_empty() {
            return Ok(vec![]);
        }

        // Convert search outputs to sequences for BM25 ranking
        let search_outputs_sequence: Vec<Vec<String>> = search_output
            .iter()
            .map(|item| {
                vec![
                    item.source.clone(),
                    item.relationship.clone(),
                    item.destination.clone(),
                    item.context.clone().unwrap_or_default(),
                ]
            })
            .collect();

        // Create BM25 scorer
        let mut scorer = bm25::Scorer::<usize>::new();
        let embedder: bm25::Embedder = bm25::EmbedderBuilder::with_avgdl(3.0) // 3 tokens per sequence
            .b(0.75) // Standard BM25 parameter
            .k1(1.2) // Standard BM25 parameter
            .build();

        // Add documents to scorer
        for (i, sequence) in search_outputs_sequence.iter().enumerate() {
            let doc = sequence.join(" ");
            let doc_embedding = embedder.embed(&doc);
            scorer.upsert(&i, doc_embedding);
        }

        // Score query
        let query_embedding = embedder.embed(query);
        let scored_results = scorer.matches(&query_embedding);

        // Convert back to RelationResult format
        let mut ranked_results: Vec<RelationResult> = scored_results
            .into_iter()
            .take(limit)
            .map(|scored| search_output[scored.id].clone())
            .collect();

        // Sort by BM25 score (highest first)
        ranked_results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        Ok(ranked_results)
    }

    pub async fn add(&self, data: &str, filters: Filters) -> Result<AddResult> {
        let entity_type_map = self.retrieve_nodes(data, filters.clone()).await?;
        let to_be_added = self
            .establish_nodes_relations(data, filters.clone(), entity_type_map.clone())
            .await?;
        let node_list = entity_type_map.keys().cloned().collect();
        let search_output = self.client.search_graph_db(node_list, None, None).await?;
        let to_be_deleted = self
            .get_delete_entities_from_search_output(search_output, data, filters.clone())
            .await?;
        let deleted_entities = self.client.delete_entities(to_be_deleted).await?;
        let added_entities = self
            .client
            .add_entities(to_be_added, &entity_type_map)
            .await?;
        Ok(AddResult {
            deleted_entities,
            added_entities,
        })
    }

    /// call the agent to extract entities from the user query
    pub async fn retrieve_nodes(
        &self,
        data: &str,
        _filters: Filters, // TODO filters can be used in a cool way given pubkeys/addresses
    ) -> Result<HashMap<String, String>> {
        let calls = extract_tool_calls(
            &get_tool_calls(
                format!("Text: {}", data),
                EXTRACT_ENTITIES_PROMPT.to_string(),
                vec![extract_entities_tool()],
            )
            .await?,
        )?;

        let mut entity_type_map = HashMap::new();

        for call in calls {
            if call["name"] == "extract_entities" {
                let entities = call["args"]["entities"].as_array().unwrap();
                for item in entities {
                    entity_type_map.insert(
                        item["entity"].as_str().unwrap().to_string(),
                        item["entity_type"].as_str().unwrap().to_string(),
                    );
                }
            }
        }

        entity_type_map = entity_type_map
            .iter()
            .map(|(key, value)| (key.replace(" ", "_"), value.replace(" ", "_")))
            .collect();

        if entity_type_map.is_empty() {
            return Ok(HashMap::new());
        }

        Ok(entity_type_map)
    }

    /// call the agent with extract_relations_tool available to take the list of
    /// entity => entity_type into list of GraphEntity (with source, relationship, destination)
    pub async fn establish_nodes_relations(
        &self,
        data: &str,
        _filters: Filters,
        entity_type_map: HashMap<String, String>,
    ) -> Result<Vec<GraphEntity>> {
        let calls = extract_tool_calls(
            &get_tool_calls(
                format!(
                    "List of entities: {}. \n\nText: {}",
                    serde_json::to_string_pretty(&entity_type_map).unwrap(),
                    data
                ),
                EXTRACT_RELATIONS_PROMPT.to_string(),
                vec![relations_tool()],
            )
            .await?,
        )?;

        let mut entities = vec![];
        for call in calls {
            if call["name"] == "establish_relationships" {
                let relations = call["args"]["entities"].as_array().unwrap();
                for item in relations {
                    let entity = serde_json::from_value::<GraphEntity>(item.clone())?;
                    entities.push(entity);
                }
            }
        }

        Ok(remove_spaces_from_entities(entities))
    }

    /// call the agent with delete_memory_tool_graph available to find the GraphEntities to trim
    pub async fn get_delete_entities_from_search_output(
        &self,
        search_output: Vec<RelationResult>,
        data: &str,
        _filters: Filters,
    ) -> Result<Vec<GraphEntity>> {
        let existing_memories = search_output
            .iter()
            .map(|item| item.stringify())
            .collect::<Vec<_>>();
        let existing_memories = serde_json::to_string(&existing_memories)?;
        let prompt = format!(
            "Existing memories: {}\n\nNew information: {}",
            existing_memories, data
        );
        println!("{}", prompt);
        let calls = extract_tool_calls(
            &get_tool_calls(
                prompt,
                DELETE_RELATIONS_PROMPT.to_string(),
                vec![delete_memory_tool_graph(), noop_tool()],
            )
            .await?,
        )?;

        let mut to_be_deleted = vec![];
        for call in calls {
            if call["name"] == "delete_graph_memory" {
                let entity = serde_json::from_value::<GraphEntity>(call["args"].clone())?;
                to_be_deleted.push(entity);
            }
        }

        Ok(remove_spaces_from_entities(to_be_deleted))
    }
}
