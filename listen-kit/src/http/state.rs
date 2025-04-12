use anyhow::Result;
use privy::Privy;
use std::sync::Arc;

use listen_memory::mem0::Mem0;
use listen_mongo::MongoClient;

pub struct AppState {
    pub(crate) privy: Arc<Privy>,
    pub(crate) mongo: Arc<MongoClient>,
    pub(crate) memory: Arc<Mem0>,
}

impl AppState {
    pub async fn new(privy: Privy, mongo: MongoClient) -> Result<Self> {
        Ok(Self {
            privy: Arc::new(privy),
            mongo: Arc::new(mongo),
            memory: Arc::new(Mem0::default()),
        })
    }
}
