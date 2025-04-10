use anyhow::Result;
use listen_mongo::{MongoClient, MongoError};

use crate::{memory_note::MemoryNote, util::must_get_env};
pub struct MemoryStore {
    pub client: MongoClient,
    pub collection_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryStoreError {
    #[error("Failed to update memory: {0}")]
    UpdateMemoryError(MongoError),
    #[error("Failed to insert memory: {0}")]
    InsertMemoryError(MongoError),
    #[error("Failed to get memory: {0}")]
    GetMemoryError(MongoError),
    #[error("Failed to create client: {0}")]
    CreateClientError(MongoError),
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

    pub async fn update_memory(&self, memory: MemoryNote) -> Result<(), MemoryStoreError> {
        self.client
            .update(&self.collection_name, &memory.id.to_string(), memory)
            .await
            .map_err(MemoryStoreError::UpdateMemoryError)?;
        Ok(())
    }

    pub async fn add_memory(&self, memory: MemoryNote) -> Result<(), MemoryStoreError> {
        self.client
            .insert_one_with_uuid(&self.collection_name, &memory.id.to_string(), memory)
            .await
            .map_err(MemoryStoreError::InsertMemoryError)?;
        Ok(())
    }

    pub async fn get_memory(&self, uuid: &str) -> Result<Option<MemoryNote>, MemoryStoreError> {
        let memory = self
            .client
            .find_by_uuid(&self.collection_name, uuid)
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
        let uuid = "5682e320-98b2-4c58-bb24-c4d377866d0e";
        let memory_store = MemoryStore::from_env().await.unwrap();
        let memory = memory_store.get_memory(uuid).await.unwrap().unwrap();
        println!("memory: {}", serde_json::to_string_pretty(&memory).unwrap());
    }
}
