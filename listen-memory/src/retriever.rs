use anyhow::Result;
use async_trait::async_trait;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, DeletePointsBuilder, Distance, PointId, PointStruct,
    ScalarQuantizationBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant, QdrantError};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::embed::generate_embedding;
use crate::util::must_get_env;

#[derive(Debug, thiserror::Error)]
pub enum RetrieverError {
    #[error("Failed to upsert point: {0}")]
    UpsertPointError(QdrantError),
    #[error("Failed to delete point: {0}")]
    DeletePointError(QdrantError),
    #[error("Failed to update point: {0}")]
    UpdatePointError(QdrantError),
    #[error("Failed to search points: {0}")]
    SearchPointsError(QdrantError),
    #[error("Failed to convert metadata to payload: {0}")]
    ConvertMetadataError(serde_json::Error),
    #[error("Failed to convert value to payload: {0}")]
    ConvertValueError(QdrantError),
    #[error("Failed to generate embedding: {0}")]
    GenerateEmbeddingError(anyhow::Error),
    #[error("Failed to create collection: {0}")]
    CreateCollectionError(QdrantError),
    #[error("Failed to check if collection exists: {0}")]
    CollectionExistsError(QdrantError),
    #[error("Failed to delete collection: {0}")]
    DeleteCollectionError(QdrantError),
    #[error("Failed to create client: {0}")]
    CreateClientError(QdrantError),
}

#[async_trait]
pub trait Retriever {
    async fn add_document(
        &self,
        document: &str,
        metadata: HashMap<String, Value>,
        doc_id: &str,
    ) -> Result<(), RetrieverError>;
    async fn delete_document(&self, doc_id: &str) -> Result<(), RetrieverError>;
    async fn update_document(
        &self,
        document: &str,
        metadata: HashMap<String, Value>,
        doc_id: &str,
    ) -> Result<(), RetrieverError>;
    async fn search(&self, query: &str, k: usize)
        -> Result<HashMap<String, Value>, RetrieverError>;
}

pub struct QdrantRetriever {
    client: Qdrant,
    collection_name: String,
}

impl QdrantRetriever {
    pub async fn from_env() -> Result<Self, RetrieverError> {
        let url = must_get_env("QDRANT_URL");
        let collection_name = must_get_env("QDRANT_COLLECTION_NAME");
        Self::new(&url, &collection_name).await
    }

    pub async fn new(url: &str, collection_name: &str) -> Result<Self, RetrieverError> {
        let client = Qdrant::from_url(url)
            .build()
            .map_err(RetrieverError::CreateClientError)?;

        if !client
            .collection_exists(&collection_name.to_string())
            .await
            .map_err(RetrieverError::CollectionExistsError)?
        {
            client
                .create_collection(
                    CreateCollectionBuilder::new(&collection_name.to_string())
                        .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine))
                        .quantization_config(ScalarQuantizationBuilder::default()),
                )
                .await
                .map_err(RetrieverError::CreateCollectionError)?;
        }

        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
        })
    }
}

#[async_trait]
impl Retriever for QdrantRetriever {
    async fn add_document(
        &self,
        document: &str,
        metadata: HashMap<String, Value>,
        doc_id: &str,
    ) -> Result<(), RetrieverError> {
        // Convert metadata to serializable format
        let mut processed_metadata = HashMap::new();
        for (key, value) in metadata {
            let processed_value = match value {
                Value::Array(_) | Value::Object(_) => Value::String(value.to_string()),
                _ => Value::String(value.to_string()),
            };
            processed_metadata.insert(key, processed_value);
        }

        // Convert to Qdrant Payload
        let payload: Payload = serde_json::to_value(processed_metadata)
            .map_err(RetrieverError::ConvertMetadataError)?
            .try_into()
            .map_err(RetrieverError::ConvertValueError)?;

        // Generate embedding for the document
        let embedding = generate_embedding(document)
            .await
            .map_err(RetrieverError::GenerateEmbeddingError)?;

        // Create point with either numeric or string ID
        let point_id: qdrant_client::qdrant::PointId = if let Ok(id) = doc_id.parse::<u64>() {
            id.into()
        } else {
            doc_id.to_string().into()
        };

        // Create point
        let point = PointStruct::new(point_id, embedding, payload);

        // Upsert point to Qdrant
        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, vec![point]))
            .await
            .map_err(RetrieverError::UpsertPointError)?;

        Ok(())
    }

    async fn delete_document(&self, doc_id: &str) -> Result<(), RetrieverError> {
        // Create point selector with either numeric or string ID
        let point_id: qdrant_client::qdrant::PointId = if let Ok(id) = doc_id.parse::<u64>() {
            id.into()
        } else {
            doc_id.to_string().into()
        };

        // Create a DeletePointsBuilder with the collection name and points
        let delete_request = DeletePointsBuilder::new(&self.collection_name).points(vec![point_id]);

        // Delete the point
        self.client
            .delete_points(delete_request)
            .await
            .map_err(RetrieverError::DeletePointError)?;

        Ok(())
    }

    async fn update_document(
        &self,
        document: &str,
        metadata: HashMap<String, Value>,
        doc_id: &str,
    ) -> Result<(), RetrieverError> {
        // First delete the existing document
        self.delete_document(doc_id).await?;

        // Then add the updated document
        self.add_document(document, metadata, doc_id).await?;

        Ok(())
    }

    async fn search(
        &self,
        query: &str,
        k: usize,
    ) -> Result<HashMap<String, Value>, RetrieverError> {
        // Generate embedding for the query
        let query_embedding = generate_embedding(query)
            .await
            .map_err(RetrieverError::GenerateEmbeddingError)?;

        // Search for similar points
        let search_result = self
            .client
            .search_points(
                SearchPointsBuilder::new(&self.collection_name, query_embedding, k as u64)
                    .with_payload(true),
            )
            .await
            .map_err(RetrieverError::SearchPointsError)?;

        // Process results into the expected format
        let mut results: HashMap<String, Value> = HashMap::new();

        let mut documents = Vec::new();
        let mut metadatas = Vec::new();
        let mut ids = Vec::new();
        let mut distances = Vec::new();

        for point in search_result.result {
            // Extract ID
            if let Some(point_id) = point.id {
                // Create simple metadata using JSON conversion to avoid type issues
                let mut simple_metadata = HashMap::<String, serde_json::Value>::new();

                // Convert point.payload to JSON string then back to avoid type mismatches
                let payload_json = serde_json::to_string(&point.payload).unwrap_or_default();
                if !payload_json.is_empty() {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&payload_json)
                    {
                        if let serde_json::Value::Object(map) = json_value {
                            simple_metadata = map.into_iter().map(|(k, v)| (k, v)).collect();
                        }
                    }
                }

                // Store results
                ids.push(point_id);
                metadatas.push(
                    serde_json::to_value(simple_metadata)
                        .map_err(RetrieverError::ConvertMetadataError)?,
                );
                distances.push(1.0 - point.score);
                documents.push(Value::String(String::from(
                    "[Document content not available]",
                )));
            }
        }

        // Format the results as expected
        results.insert(
            "ids".to_string(),
            json!(ids
                .iter()
                .map(|id| point_id_to_string(id))
                .collect::<Vec<String>>()),
        );
        results.insert("metadatas".to_string(), json!(metadatas));
        results.insert("documents".to_string(), json!(documents));
        results.insert("distances".to_string(), json!(distances));

        Ok(results)
    }
}

pub fn point_id_to_string(point_id: &PointId) -> String {
    match &point_id.point_id_options {
        Some(PointIdOptions::Num(num)) => num.to_string(),
        Some(PointIdOptions::Uuid(uuid)) => uuid.to_string(),
        _ => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_add_and_search_documents() -> Result<()> {
        let retriever = QdrantRetriever::new("http://localhost:6334", "test_collection_2").await?;

        let documents = vec![
            (
                Uuid::new_v4().to_string(),
                "This is a document about artificial intelligence",
                json!({"topic": "AI", "source": "test", "priority": "high"}),
            ),
            (
                Uuid::new_v4().to_string(),
                "Quantum computing is the future of computation",
                json!({"topic": "quantum", "source": "test", "priority": "medium"}),
            ),
            (
                Uuid::new_v4().to_string(),
                "Machine learning models need lots of data",
                json!({"topic": "ML", "source": "test", "priority": "high"}),
            ),
        ];

        for (doc_id, content, metadata) in &documents {
            let metadata_map: HashMap<String, Value> = metadata
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            retriever
                .add_document(content, metadata_map, doc_id)
                .await?;
            println!("Added document: {}", doc_id);
        }

        let query = "artificial intelligence";
        let results = retriever.search(query, 2).await?;

        assert!(results.contains_key("ids"), "Results should contain ids");
        assert!(
            results.contains_key("metadatas"),
            "Results should contain metadatas"
        );
        assert!(
            results.contains_key("documents"),
            "Results should contain documents"
        );
        assert!(
            results.contains_key("distances"),
            "Results should contain distances"
        );

        println!("Search results for '{}': {:?}", query, results);

        for (doc_id, _, _) in &documents {
            retriever.delete_document(doc_id).await?;
            println!("Deleted document: {}", doc_id);
        }

        Ok(())
    }
}
