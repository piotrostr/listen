use crate::{embed::generate_embedding, graph::EntityInfo};
use anyhow::Result;
use neo4rs::{Graph, Query};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Neo4jClient {
    pub graph: Graph,
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
// gotta trim the shitty relations, important to keep the timestamp "signal" metrics
// volume, engagement etc.

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphEntity {
    /// The canonical identifier of the source node (e.g., "user:twitter:alice")
    pub source: String,
    /// The canonical identifier of the destination node (e.g., "token:solana:xyz...")
    pub destination: String,
    /// The type of relationship between the source and destination.
    pub relationship: String,
    /// Optional timestamp associated with the relationship.
    pub timestamp: Option<String>,
    /// Optional context providing more details about the relationship.
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationResult {
    /// Canonical ID of the source node.
    pub source_canonical_id: String,
    /// Human-readable name of the source node.
    pub source_name: String,
    /// Type of relationship.
    pub relationship: String,
    /// Neo4j internal element ID of the relationship.
    pub relation_id: String,
    /// Canonical ID of the destination node.
    pub destination_canonical_id: String,
    /// Human-readable name of the destination node.
    pub destination_name: String,
    /// Relevance score (e.g., from vector index) indicating how relevant this relationship is to the query. Higher is better.
    pub relevance_score: f64,
    /// Optional timestamp of the relationship.
    pub timestamp: Option<String>,
    /// Optional context of the relationship.
    pub context: Option<String>,
    /// Number of relationships connected to the source node.
    pub source_degree: i64,
    /// Number of relationships connected to the destination node.
    pub destination_degree: i64,
}

impl RelationResult {
    pub fn stringify(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({
            "source_id": self.source_canonical_id,
            "source_name": self.source_name,
            "relationship": self.relationship,
            "destination_id": self.destination_canonical_id,
            "destination_name": self.destination_name,
            "relevance": self.relevance_score,
            "source_link_count": self.source_degree,
            "destination_link_count": self.destination_degree,
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

        let client = Self { graph };

        client.setup_constraints().await?;
        client.setup_vector_index().await?;

        Ok(client)
    }

    pub async fn setup_constraints(&self) -> Result<()> {
        let constraint_query = "
        CREATE CONSTRAINT entity_canonical_id_unique IF NOT EXISTS
        FOR (e:Entity) REQUIRE e.canonical_id IS UNIQUE";
        self.graph
            .run(Query::new(constraint_query.to_string()))
            .await?;
        println!("Constraint 'entity_canonical_id_unique' ensured.");
        Ok(())
    }

    pub async fn setup_vector_index(&self) -> Result<()> {
        let dimension = 768;
        let index_query = format!(
            r#"
            CREATE VECTOR INDEX entityEmbeddingIndex IF NOT EXISTS
            FOR (e:Entity) ON (e.embedding)
            OPTIONS {{ indexConfig: {{
                `vector.dimensions`: {},
                `vector.similarity_function`: 'cosine'
            }} }}
            "#,
            dimension
        );

        self.graph.run(Query::new(index_query)).await?;
        println!(
            "Vector index 'entityEmbeddingIndex' ensured for dimension {}.",
            dimension
        );
        Ok(())
    }

    pub async fn add_entities(
        &self,
        to_be_added: Vec<GraphEntity>,
        entity_info_map: &HashMap<String, EntityInfo>,
    ) -> Result<Vec<HashMap<String, String>>> {
        let mut results = Vec::new();

        for item in to_be_added {
            let source_info = entity_info_map.get(&item.source);
            let dest_info = entity_info_map.get(&item.destination);

            if source_info.is_none() || dest_info.is_none() {
                println!(
                    "Warning: Missing entity info for source '{}' or destination '{}'. Skipping relation.",
                    item.source, item.destination
                );
                continue;
            }

            let source_info = source_info.unwrap();
            let dest_info = dest_info.unwrap();

            let source_text_for_embedding = format!(
                "Entity ID: {} Name: {} Type: {} Context: {}",
                item.source,
                source_info.name,
                source_info.entity_type,
                item.context.as_deref().unwrap_or("")
            );
            let dest_text_for_embedding = format!(
                "Entity ID: {} Name: {} Type: {} Context: {}",
                item.destination,
                dest_info.name,
                dest_info.entity_type,
                item.context.as_deref().unwrap_or("")
            );

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

            let cypher = format!(
                r#"
                MERGE (source:Entity:{source_type} {{canonical_id: $source_id}})
                ON CREATE SET
                    source.name = $source_name,
                    source.entity_type = $source_type,
                    source.created_at = timestamp(),
                    source.updated_at = timestamp(),
                    source.embedding = $source_embedding
                ON MATCH SET
                    source.name = $source_name,
                    source.entity_type = $source_type,
                    source.updated_at = timestamp(),
                    source.embedding = $source_embedding

                MERGE (destination:Entity:{destination_type} {{canonical_id: $dest_id}})
                ON CREATE SET
                    destination.name = $dest_name,
                    destination.entity_type = $destination_type,
                    destination.created_at = timestamp(),
                    destination.updated_at = timestamp(),
                    destination.embedding = $dest_embedding
                ON MATCH SET
                    destination.name = $dest_name,
                    destination.entity_type = $destination_type,
                    destination.updated_at = timestamp(),
                    destination.embedding = $dest_embedding

                MERGE (source)-[rel:{relationship}]->(destination)
                ON CREATE SET
                    rel.created_at = timestamp(),
                    rel.updated_at = timestamp(),
                    rel.timestamp = $timestamp,
                    rel.context = $context
                ON MATCH SET
                    rel.updated_at = timestamp(),
                    rel.timestamp = $timestamp,
                    rel.context = $context

                RETURN
                    source.canonical_id AS source_id,
                    type(rel) AS relationship,
                    destination.canonical_id AS destination_id
                "#,
                source_type = source_info.entity_type,
                destination_type = dest_info.entity_type,
                relationship = item.relationship
            );

            let query = Query::new(cypher)
                .param("source_id", item.source.clone())
                .param("source_name", source_info.name.clone())
                .param("source_type", source_info.entity_type.clone())
                .param("source_embedding", source_embedding)
                .param("dest_id", item.destination.clone())
                .param("dest_name", dest_info.name.clone())
                .param("destination_type", dest_info.entity_type.clone())
                .param("dest_embedding", dest_embedding)
                .param("timestamp", item.timestamp.clone().unwrap_or_default())
                .param("context", item.context.clone().unwrap_or_default());

            let mut result = self.graph.execute(query).await?;

            let mut row_result = HashMap::new();
            if let Some(row) = result.next().await? {
                row_result.insert(
                    "source".to_string(),
                    row.get::<String>("source_id").unwrap(),
                );
                row_result.insert(
                    "relationship".to_string(),
                    row.get::<String>("relationship").unwrap(),
                );
                row_result.insert(
                    "target".to_string(),
                    row.get::<String>("destination_id").unwrap(),
                );
            } else {
                println!(
                    "Warning: No result returned for adding relation: {} -[{}]-> {}",
                    item.source, item.relationship, item.destination
                );
            }
            results.push(row_result);
        }

        Ok(results)
    }

    pub async fn get_all_entities(&self) -> Result<Vec<GraphEntity>> {
        let cypher = r#"
        MATCH (n:Entity)-[r]->(m:Entity)
        RETURN
            n.canonical_id AS source,
            type(r) AS relationship,
            m.canonical_id AS destination,
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
        query_text: String,
        threshold: Option<f64>,
        limit: Option<usize>,
    ) -> Result<Vec<RelationResult>> {
        let threshold = threshold.unwrap_or(0.7);
        let limit = limit.unwrap_or(15);
        let candidate_limit = limit + 5;
        let mut result_relations = Vec::new();

        let query_embedding: Vec<f64> = generate_embedding(&query_text)
            .await?
            .into_iter()
            .map(|x| x as f64)
            .collect();

        let cypher_query = r#"
            CALL db.index.vector.queryNodes('entityEmbeddingIndex', $candidate_limit, $query_embedding)
            YIELD node AS n, score AS score
            WHERE score >= $threshold
            WITH n, score ORDER BY score DESC LIMIT $candidate_limit
            WITH collect({node: n, score: score}) AS top_candidates

            UNWIND top_candidates AS candidate_data
            WITH candidate_data.node AS n, candidate_data.score AS score

            MATCH (n)-[r]-(m:Entity)

            WITH n, r, m, score, startNode(r) AS srcNode, endNode(r) AS destNode

            OPTIONAL MATCH (srcNode)-[srcRel]-()
            WITH n, r, m, score, srcNode, destNode, count(DISTINCT srcRel) AS srcDegree
            OPTIONAL MATCH (destNode)-[destRel]-()
            WITH n, r, m, score, srcNode, destNode, srcDegree, count(DISTINCT destRel) AS destDegree

            ORDER BY score DESC

            RETURN DISTINCT
                srcNode.canonical_id AS source_canonical_id,
                srcNode.name AS source_name,
                type(r) AS relationship,
                elementId(r) AS relation_id,
                destNode.canonical_id AS destination_canonical_id,
                destNode.name AS destination_name,
                score AS relevance_score,
                r.timestamp AS timestamp,
                r.context AS context,
                srcDegree AS source_degree,
                destDegree AS destination_degree
            LIMIT $limit
            "#;

        let mut result = self
            .graph
            .execute(
                Query::new(cypher_query.to_string())
                    .param("query_embedding", query_embedding)
                    .param("threshold", threshold)
                    .param("candidate_limit", candidate_limit as i64)
                    .param("limit", limit as i64),
            )
            .await?;

        while let Some(row) = result.next().await? {
            result_relations.push(RelationResult {
                source_canonical_id: row.get("source_canonical_id").unwrap_or_default(),
                source_name: row.get("source_name").unwrap_or_default(),
                relationship: row.get("relationship").unwrap_or_default(),
                relation_id: row.get("relation_id").unwrap_or_default(),
                destination_canonical_id: row.get("destination_canonical_id").unwrap_or_default(),
                destination_name: row.get("destination_name").unwrap_or_default(),
                relevance_score: row.get("relevance_score").unwrap_or(0.0),
                timestamp: row.get("timestamp").ok(),
                context: row.get("context").ok(),
                source_degree: row.get("source_degree").unwrap_or(0),
                destination_degree: row.get("destination_degree").unwrap_or(0),
            });
        }

        Ok(result_relations)
    }
}
