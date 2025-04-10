use anyhow::Result;
use listen_mongo::MongoClient;

use crate::{memory_note::MemoryNote, util::must_get_env};
pub struct MemoryStore {
    pub client: MongoClient,
    pub collection_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryStoreError {
    #[error("Failed to update memory: {0}")]
    UpdateMemoryError(anyhow::Error),
    #[error("Failed to insert memory: {0}")]
    InsertMemoryError(anyhow::Error),
    #[error("Failed to get memory: {0}")]
    GetMemoryError(anyhow::Error),
    #[error("Failed to create client: {0}")]
    CreateClientError(anyhow::Error),
}

impl MemoryStore {
    pub async fn from_env() -> Result<Self, MemoryStoreError> {
        let client = MongoClient::with_config(
            must_get_env("MONGODB_URI"),
            must_get_env("MONGODB_MEMORY_DB_NAME"),
        )
        .await
        .map_err(MemoryStoreError::CreateClientError)?;

        let collection_name = must_get_env("MONGODB_MEMORY_COLLECTION_NAME");
        Ok(Self {
            client,
            collection_name,
        })
    }

    pub async fn update_memory(
        &self,
        id: &str,
        memory: MemoryNote,
    ) -> Result<(), MemoryStoreError> {
        self.client
            .update(&self.collection_name, id, memory)
            .await
            .map_err(MemoryStoreError::UpdateMemoryError)?;
        Ok(())
    }

    pub async fn add_memory(&self, memory: MemoryNote) -> Result<(), MemoryStoreError> {
        self.client
            .insert_one_with_id(&self.collection_name, &memory.id.to_string(), memory)
            .await
            .map_err(MemoryStoreError::InsertMemoryError)?;
        Ok(())
    }

    pub async fn get_memory(&self, id: &str) -> Result<Option<MemoryNote>, MemoryStoreError> {
        let memory = self
            .client
            .find_by_id(&self.collection_name, id)
            .await
            .map_err(MemoryStoreError::GetMemoryError)?;
        Ok(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_doc() {
        dotenv::dotenv().ok();
        let id = "8d815c69-d07e-42a1-ba67-f5c358349538";
        let memory_store = MemoryStore::from_env().await.unwrap();
        let memory = memory_store.get_memory(id).await.unwrap().unwrap();
        println!("memory: {}", serde_json::to_string_pretty(&memory).unwrap());
    }

    #[tokio::test]
    async fn test_list_docs() {
        dotenv::dotenv().ok();
        let mongo = MemoryStore::from_env().await.unwrap().client;
        let docs: Vec<MemoryNote> = mongo.list_docs("memories").await.unwrap();
        println!("docs: {}", serde_json::to_string_pretty(&docs).unwrap());
    }
}
