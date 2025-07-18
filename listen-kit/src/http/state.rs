use anyhow::Result;
use privy::Privy;
use std::sync::Arc;

use listen_memory::graph::GraphMemory;
use listen_mongo::MongoClient;

pub struct AppState {
    pub(crate) privy: Arc<Privy>,
    pub(crate) mongo: Option<Arc<MongoClient>>,
    pub(crate) global_memory: Option<Arc<GraphMemory>>,
}

impl AppState {
    pub async fn new(
        privy: Privy,
        mongo: Option<Arc<MongoClient>>,
        global_memory: Option<Arc<GraphMemory>>,
    ) -> Result<Self> {
        Ok(Self {
            privy: Arc::new(privy),
            mongo,
            global_memory,
        })
    }
}
