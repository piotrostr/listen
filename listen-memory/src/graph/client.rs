use crate::embed::generate_embedding;
use anyhow::Result;
use neo4rs::{Graph, Query};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Neo4jClient {
    graph: Graph,
}

// TODO implement entity types, there are only a few entity IDs really:
// X username
// X Post ID
// website URL
// address - any chain token, program, wallet
// -> use canonical identifiers
// username:{platform}:{username}
// post:{platform}:{post_id} -> X post ID, telegram post, farcaster etc
// url:{url}
// address:{chain}:{address} -> token, program, wallet, account, LP, NFT etc.
//
// it also makes sense to track the link count, this increases relevance
// and stands for sentiment analysis
//
// gotta trim the shitty relations, important to keep the timestamp "signal" metrics
// volume, engagement etc.

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEntity {
    pub source: String,
    pub destination: String,
    pub relationship: String,
    pub timestamp: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationResult {
    pub source: String,
    pub source_id: String,
    pub relationship: String,
    pub relation_id: String,
    pub destination: String,
    pub destination_id: String,
    pub similarity: f64,
    pub timestamp: Option<String>,
    pub context: Option<String>,
}

impl RelationResult {
    pub fn stringify(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({
            "source": self.source,
            "relationship": self.relationship,
            "destination": self.destination,
            "similarity": self.similarity,
        });
        if let Some(timestamp) = self.timestamp.clone() {
            obj["timestamp"] = serde_json::Value::String(timestamp);
        }
        if let Some(context) = self.context.clone() {
            obj["context"] = serde_json::Value::String(context);
        }
        obj
    }
}

impl Neo4jClient {
    pub async fn from_env() -> Result<Self> {
        let graph = Graph::new("bolt://localhost:7687", "neo4j", "password").await?;
        Ok(Self { graph })
    }

    pub async fn search_source_node(
        &self,
        source_embedding: Vec<f64>,
        threshold: Option<f64>,
    ) -> Result<Option<String>> {
        let threshold = threshold.unwrap_or(0.9);

        let cypher = r#"
            MATCH (source_candidate)
            WHERE source_candidate.embedding IS NOT NULL 

            WITH source_candidate,
                round(
                    reduce(dot = 0.0, i IN range(0, size(source_candidate.embedding)-1) |
                        dot + source_candidate.embedding[i] * $source_embedding[i]) /
                    (sqrt(reduce(l2 = 0.0, i IN range(0, size(source_candidate.embedding)-1) |
                        l2 + source_candidate.embedding[i] * source_candidate.embedding[i])) *
                    sqrt(reduce(l2 = 0.0, i IN range(0, size($source_embedding)-1) |
                        l2 + $source_embedding[i] * $source_embedding[i])))
                , 4) AS source_similarity
            WHERE source_similarity >= $threshold

            WITH source_candidate, source_similarity
            ORDER BY source_similarity DESC
            LIMIT 1

            RETURN elementId(source_candidate)
        "#;

        let mut result = self
            .graph
            .execute(
                Query::new(cypher.to_string())
                    .param("source_embedding", source_embedding)
                    .param("threshold", threshold),
            )
            .await?;

        if let Some(row) = result.next().await? {
            let id: String = row.get("elementId(source_candidate)").unwrap();
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    pub async fn search_destination_node(
        &self,
        destination_embedding: Vec<f64>,
        threshold: Option<f64>,
    ) -> Result<Option<String>> {
        let threshold = threshold.unwrap_or(0.9);

        let cypher = r#"
            MATCH (destination_candidate)
            WHERE destination_candidate.embedding IS NOT NULL 

            WITH destination_candidate,
                round(
                    reduce(dot = 0.0, i IN range(0, size(destination_candidate.embedding)-1) |
                        dot + destination_candidate.embedding[i] * $destination_embedding[i]) /
                    (sqrt(reduce(l2 = 0.0, i IN range(0, size(destination_candidate.embedding)-1) |
                        l2 + destination_candidate.embedding[i] * destination_candidate.embedding[i])) *
                    sqrt(reduce(l2 = 0.0, i IN range(0, size($destination_embedding)-1) |
                        l2 + $destination_embedding[i] * $destination_embedding[i])))
                , 4) AS destination_similarity
            WHERE destination_similarity >= $threshold

            WITH destination_candidate, destination_similarity
            ORDER BY destination_similarity DESC
            LIMIT 1

            RETURN elementId(destination_candidate)
        "#;

        let mut result = self
            .graph
            .execute(
                Query::new(cypher.to_string())
                    .param("destination_embedding", destination_embedding)
                    .param("threshold", threshold),
            )
            .await?;

        if let Some(row) = result.next().await? {
            let id: String = row.get("elementId(destination_candidate)").unwrap();
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    pub async fn add_entities(
        &self,
        to_be_added: Vec<GraphEntity>,
        entity_type_map: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>> {
        let mut results = Vec::new();

        for item in to_be_added {
            let source = item.source;
            let destination = item.destination;
            let relationship = item.relationship;

            let source_type = entity_type_map
                .get(&source)
                .unwrap_or(&"unknown".to_string())
                .to_string();
            let destination_type = entity_type_map
                .get(&destination)
                .unwrap_or(&"unknown".to_string())
                .to_string();

            // Combine name and context for embedding generation
            let source_text_for_embedding = match &item.context {
                Some(ctx) if !ctx.is_empty() => format!("{} {}", source, ctx),
                _ => source.clone(),
            };
            let dest_text_for_embedding = match &item.context {
                Some(ctx) if !ctx.is_empty() => format!("{} {}", destination, ctx),
                _ => destination.clone(),
            };

            // Generate embeddings using combined text
            let source_embedding: Vec<f64> = generate_embedding(&source_text_for_embedding)
                .await?
                .into_iter()
                .map(|x| x as f64)
                .collect();
            let dest_embedding: Vec<f64> = generate_embedding(&dest_text_for_embedding)
                .await?
                .into_iter()
                .map(|x| x as f64)
                .collect();

            // Search for existing nodes using the new context-aware embeddings
            let source_node = self
                .search_source_node(source_embedding.clone(), Some(0.9))
                .await?;
            let destination_node = self
                .search_destination_node(dest_embedding.clone(), Some(0.9))
                .await?;

            let query = match (source_node, destination_node) {
                (None, Some(dest_id)) => {
                    let cypher = format!(
                        r#"
                        MATCH (destination)
                        WHERE elementId(destination) = $destination_id
                        MERGE (source:{} {{name: $source_name}})
                        ON CREATE SET
                            source.created = timestamp(),
                            source.embedding = $source_embedding
                        MERGE (source)-[r:{}]->(destination)
                        ON CREATE SET 
                            r.created = timestamp()
                        RETURN source.name AS source, type(r) AS relationship, destination.name AS target
                        "#,
                        source_type, relationship
                    );

                    Query::new(cypher)
                        .param("destination_id", dest_id)
                        .param("source_name", source)
                        .param("source_embedding", source_embedding)
                }
                (Some(source_id), None) => {
                    let cypher = format!(
                        r#"
                        MATCH (source)
                        WHERE elementId(source) = $source_id
                        MERGE (destination:{} {{name: $destination_name}})
                        ON CREATE SET
                            destination.created = timestamp(),
                            destination.embedding = $destination_embedding
                        MERGE (source)-[r:{}]->(destination)
                        ON CREATE SET 
                            r.created = timestamp()
                        RETURN source.name AS source, type(r) AS relationship, destination.name AS target
                        "#,
                        destination_type, relationship
                    );

                    Query::new(cypher)
                        .param("source_id", source_id)
                        .param("destination_name", destination)
                        .param("destination_embedding", dest_embedding)
                }
                (Some(source_id), Some(dest_id)) => {
                    let cypher = format!(
                        r#"
                        MATCH (source)
                        WHERE elementId(source) = $source_id
                        MATCH (destination)
                        WHERE elementId(destination) = $destination_id
                        MERGE (source)-[r:{}]->(destination)
                        ON CREATE SET 
                            r.created_at = timestamp(),
                            r.updated_at = timestamp()
                        RETURN source.name AS source, type(r) AS relationship, destination.name AS target
                        "#,
                        relationship
                    );

                    Query::new(cypher)
                        .param("source_id", source_id)
                        .param("destination_id", dest_id)
                }
                (None, None) => {
                    let cypher = format!(
                        r#"
                        MERGE (n:{} {{name: $source_name}})
                        ON CREATE SET 
                            n.created = timestamp(), 
                            n.embedding = $source_embedding,
                            n.context = $context
                        ON MATCH SET n.embedding = $source_embedding
                        MERGE (m:{} {{name: $dest_name}})
                        ON CREATE SET 
                            m.created = timestamp(), 
                            m.embedding = $dest_embedding,
                            m.context = $context
                        ON MATCH SET m.embedding = $dest_embedding
                        MERGE (n)-[rel:{}]->(m)
                        ON CREATE SET 
                            rel.created = timestamp(),
                            rel.timestamp = $timestamp,
                            rel.context = $context
                        RETURN n.name AS source, type(rel) AS relationship, m.name AS target
                        "#,
                        source_type, destination_type, relationship
                    );

                    Query::new(cypher)
                        .param("source_name", source)
                        .param("dest_name", destination)
                        .param("source_embedding", source_embedding)
                        .param("dest_embedding", dest_embedding)
                        .param(
                            "timestamp",
                            item.timestamp.unwrap_or_else(|| "".to_string()),
                        )
                        .param("context", item.context.unwrap_or_else(|| "".to_string()))
                }
            };

            let mut result = self.graph.execute(query).await?;

            let mut row_result = HashMap::new();
            if let Some(row) = result.next().await? {
                row_result.insert("source".to_string(), row.get::<String>("source").unwrap());
                row_result.insert(
                    "relationship".to_string(),
                    row.get::<String>("relationship").unwrap(),
                );
                row_result.insert("target".to_string(), row.get::<String>("target").unwrap());
            }
            results.push(row_result);
        }

        Ok(results)
    }

    pub async fn delete_entities(
        &self,
        to_be_deleted: Vec<GraphEntity>,
    ) -> Result<Vec<HashMap<String, String>>> {
        let mut results = Vec::new();
        let to_be_deleted = remove_spaces_from_entities(to_be_deleted);

        for item in to_be_deleted {
            let cypher = format!(
                r#"
                OPTIONAL MATCH (n {{name: $source_name}})
                -[r:{}]->
                (m {{name: $dest_name}})
                WHERE (r.timestamp = $timestamp OR $timestamp IS NULL)
                  AND (r.context = $context OR $context IS NULL)
                WITH n, r, m // Carry forward matched entities
                WHERE r IS NOT NULL // Ensure relationship was actually found
                WITH n.name AS source, m.name AS target, type(r) AS relationship, r.timestamp AS timestamp, r.context AS context, r AS rel_to_delete
                DELETE rel_to_delete // Delete the relationship
                RETURN source, target, relationship, timestamp, context // Return the details
                "#,
                item.relationship
            );

            let mut result = self
                .graph
                .execute(
                    Query::new(cypher)
                        .param("source_name", item.source)
                        .param("dest_name", item.destination)
                        .param("timestamp", item.timestamp)
                        .param("context", item.context),
                )
                .await?;

            let mut row_result = HashMap::new();
            if let Some(row) = result.next().await? {
                row_result.insert("source".to_string(), row.get::<String>("source").unwrap());
                row_result.insert(
                    "relationship".to_string(),
                    row.get::<String>("relationship").unwrap(),
                );
                row_result.insert("target".to_string(), row.get::<String>("target").unwrap());
                if let Ok(timestamp) = row.get::<String>("timestamp") {
                    row_result.insert("timestamp".to_string(), timestamp);
                }
                if let Ok(context) = row.get::<String>("context") {
                    row_result.insert("context".to_string(), context);
                }
            }
            results.push(row_result);
        }

        Ok(results)
    }

    pub async fn get_all_entities(&self) -> Result<Vec<GraphEntity>> {
        let cypher = r#"
        MATCH (n)-[r]->(m)
        RETURN 
            n.name AS source, 
            type(r) AS relationship, 
            m.name AS destination,
            r.timestamp AS timestamp,
            r.context AS context
        "#;

        let mut result = self.graph.execute(Query::new(cypher.to_string())).await?;

        let mut entities = Vec::new();
        while let Some(row) = result.next().await? {
            entities.push(GraphEntity {
                source: row.get::<String>("source").unwrap(),
                destination: row.get::<String>("destination").unwrap(),
                relationship: row.get::<String>("relationship").unwrap(),
                timestamp: row.get::<Option<String>>("timestamp").unwrap_or(None),
                context: row.get::<Option<String>>("context").unwrap_or(None),
            });
        }

        Ok(entities)
    }

    pub async fn search_graph_db(
        &self,
        query: String,
        threshold: Option<f64>,
        limit: Option<usize>,
    ) -> Result<Vec<RelationResult>> {
        let threshold = threshold.unwrap_or(0.5);
        let limit = limit.unwrap_or(15);
        let mut result_relations = Vec::new();

        let n_embedding: Vec<f64> = generate_embedding(&query)
            .await?
            .into_iter()
            .map(|x| x as f64)
            .collect();

        let cypher_query = r#"
            MATCH (n)
            WHERE n.embedding IS NOT NULL
            WITH n,
                round(reduce(dot = 0.0, i IN range(0, size(n.embedding)-1) | dot + n.embedding[i] * $n_embedding[i]) /
                (sqrt(reduce(l2 = 0.0, i IN range(0, size(n.embedding)-1) | l2 + n.embedding[i] * n.embedding[i])) *
                sqrt(reduce(l2 = 0.0, i IN range(0, size($n_embedding)-1) | l2 + $n_embedding[i] * $n_embedding[i]))), 4) AS similarity
            WHERE similarity >= $threshold
            MATCH (n)-[r]->(m)
            RETURN 
                n.name AS source, 
                elementId(n) AS source_id, 
                type(r) AS relationship, 
                elementId(r) AS relation_id, 
                m.name AS destination, 
                elementId(m) AS destination_id, 
                similarity,
                r.timestamp AS timestamp,
                r.context AS context
            UNION
            MATCH (n)
            WHERE n.embedding IS NOT NULL
            WITH n,
                round(reduce(dot = 0.0, i IN range(0, size(n.embedding)-1) | dot + n.embedding[i] * $n_embedding[i]) /
                (sqrt(reduce(l2 = 0.0, i IN range(0, size(n.embedding)-1) | l2 + n.embedding[i] * n.embedding[i])) *
                sqrt(reduce(l2 = 0.0, i IN range(0, size($n_embedding)-1) | l2 + $n_embedding[i] * $n_embedding[i]))), 4) AS similarity
            WHERE similarity >= $threshold
            MATCH (m)-[r]->(n)
            RETURN 
                m.name AS source, 
                elementId(m) AS source_id, 
                type(r) AS relationship, 
                elementId(r) AS relation_id, 
                n.name AS destination, 
                elementId(n) AS destination_id, 
                similarity,
                r.timestamp AS timestamp,
                r.context AS context
            "#;

        let mut result = self
            .graph
            .execute(
                Query::new(cypher_query.to_string())
                    .param("n_embedding", n_embedding)
                    .param("threshold", threshold),
            )
            .await?;

        while let Some(row) = result.next().await? {
            result_relations.push(RelationResult {
                source: row.get::<&str>("source").unwrap().to_owned(),
                source_id: row.get::<String>("source_id").unwrap(),
                relationship: row.get::<&str>("relationship").unwrap().to_owned(),
                relation_id: row.get::<String>("relation_id").unwrap(),
                destination: row.get::<&str>("destination").unwrap().to_owned(),
                destination_id: row.get::<String>("destination_id").unwrap(),
                similarity: row.get::<f64>("similarity").unwrap(),
                timestamp: row.get::<Option<String>>("timestamp").unwrap_or(None),
                context: row.get::<Option<String>>("context").unwrap_or(None),
            });
        }

        // Sort results by similarity descending
        result_relations.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncate results to the specified limit
        result_relations.truncate(limit);

        Ok(result_relations)
    }
}

pub fn remove_spaces_from_entities(entity_list: Vec<GraphEntity>) -> Vec<GraphEntity> {
    // TODO here it would be good to prevent slashes and other special characters too
    let mut entity_list = entity_list;
    for item in entity_list.iter_mut() {
        item.source = replace_special_characters(&item.source);
        item.destination = replace_special_characters(&item.destination);
        item.relationship = replace_special_characters(&item.relationship);
    }
    entity_list
}

fn replace_special_characters(text: &str) -> String {
    text.replace(" ", "_")
        .replace("(", "_")
        .replace(")", "_")
        .replace("-", "_")
}
