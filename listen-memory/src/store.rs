use anyhow::Result;
use listen_mongo::MongoClient;

use crate::memory_note::MemoryNote;
pub struct MemoryStore {
    client: MongoClient,
}

const COLLECTION_NAME: &str = "memories";

impl MemoryStore {
    pub async fn from_env() -> Result<Self> {
        let client = MongoClient::from_env().await?;
        Ok(Self { client })
    }

    pub async fn update_memory(&self, id: &str, memory: MemoryNote) -> Result<()> {
        self.client.update(COLLECTION_NAME, id, memory).await?;
        Ok(())
    }

    pub async fn add_memory(&self, memory: MemoryNote) -> Result<()> {
        self.client
            .insert_one_with_id(COLLECTION_NAME, &memory.id.to_string(), memory)
            .await?;
        Ok(())
    }

    pub async fn get_memory(&self, id: &str) -> Result<Option<MemoryNote>> {
        let memory = self.client.find_by_id(COLLECTION_NAME, id).await?;
        Ok(memory)
    }
}
