use anyhow::Result;
use listen_mongo::MongoClient;

use crate::{memory_note::MemoryNote, util::must_get_env};
pub struct MemoryStore {
    client: MongoClient,
    collection_name: String,
}

impl MemoryStore {
    pub async fn from_env() -> Result<Self> {
        let client = MongoClient::from_env().await?;
        let collection_name = must_get_env("MONGODB_COLLECTION_NAME");
        Ok(Self {
            client,
            collection_name,
        })
    }

    pub async fn update_memory(&self, id: &str, memory: MemoryNote) -> Result<()> {
        self.client
            .update(&self.collection_name, id, memory)
            .await?;
        Ok(())
    }

    pub async fn add_memory(&self, memory: MemoryNote) -> Result<()> {
        self.client
            .insert_one_with_id(&self.collection_name, &memory.id.to_string(), memory)
            .await?;
        Ok(())
    }

    pub async fn get_memory(&self, id: &str) -> Result<Option<MemoryNote>> {
        let memory = self.client.find_by_id(&self.collection_name, id).await?;
        Ok(memory)
    }
}
