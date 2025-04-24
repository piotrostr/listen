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
    client::RelationResult, prompts::EXTRACT_RELATIONS_PROMPT, tools::relations_tool,
};

pub struct GraphMemory {
    pub client: Neo4jClient,
}

impl GraphMemory {
    pub async fn from_env() -> Result<Self> {
        let client = Neo4jClient::from_env().await?;
        Ok(Self { client })
    }
}

#[derive(Debug, Clone)]
pub struct Filters {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub name: String,
    pub entity_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddResult {
    pub added_entities: Vec<HashMap<String, String>>,
}

impl GraphMemory {
    #[timed::timed]
    pub async fn search(
        &self,
        query: &str,
        _filters: Filters,
        limit: Option<usize>,
    ) -> Result<Vec<RelationResult>> {
        println!("Searching graph for: {}", query);
        let limit = limit.unwrap_or(15);

        let search_output = self
            .client
            .search_graph_db(query.to_string(), None, Some(limit))
            .await?;

        Ok(search_output)
    }

    pub async fn add(&self, data: &str, filters: Filters) -> Result<AddResult> {
        let entity_info_map = self.retrieve_nodes(data, filters.clone()).await?;

        if entity_info_map.is_empty() {
            println!("No verifiable entities extracted from data. Nothing to add.");
            return Ok(AddResult {
                added_entities: vec![],
            });
        }

        let to_be_added = self
            .establish_nodes_relations(data, filters.clone(), entity_info_map.clone())
            .await?;

        if to_be_added.is_empty() {
            println!("No verifiable relationships established between entities. Nothing to add.");
            return Ok(AddResult {
                added_entities: vec![],
            });
        }

        let added_entities = self
            .client
            .add_entities(to_be_added, &entity_info_map)
            .await?;

        Ok(AddResult { added_entities })
    }

    pub async fn retrieve_nodes(
        &self,
        data: &str,
        _filters: Filters,
    ) -> Result<HashMap<String, EntityInfo>> {
        let calls = extract_tool_calls(
            &get_tool_calls(
                format!("Text: {}", data),
                EXTRACT_ENTITIES_PROMPT.to_string(),
                vec![extract_entities_tool()],
            )
            .await?,
        )?;

        let mut entity_info_map = HashMap::new();

        for call in calls {
            if call["name"] == "extract_entities" {
                if let Some(entities) = call["args"]["entities"].as_array() {
                    for item in entities {
                        if let (Some(canonical_id), Some(name), Some(entity_type)) = (
                            item["canonical_id"].as_str(),
                            item["name"].as_str(),
                            item["entity_type"].as_str(),
                        ) {
                            let canonical_id = canonical_id.trim().to_string();
                            let name = name.trim().to_string();
                            let entity_type = entity_type.trim().to_string();

                            if !canonical_id.is_empty() && canonical_id.contains(':') {
                                entity_info_map
                                    .insert(canonical_id, EntityInfo { name, entity_type });
                            } else {
                                println!("Warning: Skipping entity with invalid canonical_id format: {:?}", item);
                            }
                        }
                    }
                }
            }
        }

        if entity_info_map.is_empty() {
            println!("Warning: No entities extracted from data.");
        }

        Ok(entity_info_map)
    }

    pub async fn establish_nodes_relations(
        &self,
        data: &str,
        _filters: Filters,
        entity_info_map: HashMap<String, EntityInfo>,
    ) -> Result<Vec<GraphEntity>> {
        let entity_list_for_prompt: Vec<_> = entity_info_map
            .iter()
            .map(|(id, info)| {
                serde_json::json!({
                    "canonical_id": id,
                    "name": info.name,
                    "type": info.entity_type
                })
            })
            .collect();

        let calls = extract_tool_calls(
            &get_tool_calls(
                format!(
                    "List of entities: {}. \n\nText: {}",
                    serde_json::to_string_pretty(&entity_list_for_prompt)
                        .unwrap_or_else(|_| "[]".to_string()),
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
                if let Some(relations) = call["args"]["entities"].as_array() {
                    for item in relations {
                        match serde_json::from_value::<GraphEntity>(item.clone()) {
                            Ok(mut entity) => {
                                entity.source = entity.source.trim().to_string();
                                entity.destination = entity.destination.trim().to_string();
                                if entity.source.contains(':') && entity.destination.contains(':') {
                                    entities.push(entity);
                                } else {
                                    println!("Warning: Skipping relation with invalid canonical_id format: {:?}", item);
                                }
                            }
                            Err(e) => {
                                println!("Error deserializing relation: {:?}, Error: {}", item, e);
                            }
                        }
                    }
                }
            }
        }
        Ok(entities)
    }
}
