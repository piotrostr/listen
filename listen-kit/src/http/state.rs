use anyhow::Result;
use privy::Privy;
use std::sync::Arc;

use listen_memory::memory_system::MemorySystem;
use listen_mongo::MongoClient;

pub struct AppState {
    pub(crate) privy: Arc<Privy>,
    pub(crate) mongo: Arc<MongoClient>,
    pub(crate) memory: Arc<MemorySystem>,
}

impl AppState {
    pub async fn new(privy: Privy, mongo: MongoClient) -> Result<Self> {
        Ok(Self {
            privy: Arc::new(privy),
            mongo: Arc::new(mongo),
            memory: Arc::new(MemorySystem::from_env().await?),
        })
    }
}
