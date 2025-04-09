use anyhow::Result;
use privy::Privy;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use listen_memory::memory_system::MemorySystem;
use listen_mongo::MongoClient;

pub struct AppState {
    pub(crate) privy: Arc<Privy>,
    pub(crate) mongo: Arc<MongoClient>,
    pub(crate) memory: Arc<MemorySystem>,
    pub(crate) dedup_set: Arc<RwLock<HashSet<String>>>, // temporary, will be replaced by redis
}

impl AppState {
    pub async fn new(privy: Privy, mongo: MongoClient) -> Result<Self> {
        Ok(Self {
            privy: Arc::new(privy),
            mongo: Arc::new(mongo),
            memory: Arc::new(MemorySystem::from_env().await?),
            dedup_set: Arc::new(RwLock::new(HashSet::new())),
        })
    }
}
