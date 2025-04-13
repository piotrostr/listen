pub mod agent;
pub mod client;
pub mod prompts;
pub mod tools;

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
    tools::{delete_memory_tool_graph, relations_tool},
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
    pub async fn add(&self, data: &str, filters: Filters) -> Result<AddResult> {
        let entity_type_map = self.retrieve_nodes(data, filters.clone()).await?;
        let to_be_added = self
            .establish_nodes_relations_from_data(data, filters.clone(), entity_type_map.clone())
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

    pub async fn retrieve_nodes(
        &self,
        data: &str,
        filters: Filters, // TODO filters can be used in a cool way given pubkeys/addresses
    ) -> Result<HashMap<String, String>> {
        let calls = extract_tool_calls(
            &get_tool_calls(
                data.to_string(),
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
            return Err(anyhow::anyhow!("No entities found"));
        }

        Ok(entity_type_map)
    }

    pub async fn establish_nodes_relations_from_data(
        &self,
        data: &str,
        filters: Filters,
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

    pub async fn get_delete_entities_from_search_output(
        &self,
        search_output: Vec<RelationResult>,
        data: &str,
        filters: Filters,
    ) -> Result<Vec<GraphEntity>> {
        let existing_memories = serde_json::to_string_pretty(&search_output)?;
        let calls = extract_tool_calls(
            &get_tool_calls(
                format!(
                    "Existing memories: {}\n\nNew information: {}",
                    existing_memories, data
                ),
                DELETE_RELATIONS_PROMPT.to_string(),
                vec![delete_memory_tool_graph()],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add() {
        let data = "Paris is the capital of France, France is west of Germany";
        let graph_memory = GraphMemory::from_env().await.unwrap();
        let result = graph_memory.add(data, Filters {}).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }

    #[tokio::test]
    async fn test_retrieve_nodes() {
        let data = "Paris is the capital of France, France is west of Germany";
        let graph_memory = GraphMemory::from_env().await.unwrap();
        let map = graph_memory.retrieve_nodes(data, Filters {}).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&map).unwrap());
    }

    #[tokio::test]
    async fn test_establish_nodes_relations_from_data() {
        let data = "Paris is the capital of France, France is west of Germany";
        let graph_memory = GraphMemory::from_env().await.unwrap();
        let entity_type_map: HashMap<String, String> = serde_json::from_str(
            r#"
            {
                "Paris": "city",
                "France": "country",
                "Germany": "country"
            }
            "#,
        )
        .unwrap();
        let graph_entities = graph_memory
            .establish_nodes_relations_from_data(data, Filters {}, entity_type_map)
            .await
            .unwrap();
        println!("{}", serde_json::to_string_pretty(&graph_entities).unwrap());
    }
}
