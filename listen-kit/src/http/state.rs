use privy::Privy;
use std::sync::Arc;

use crate::mongo::MongoClient;

pub struct AppState {
    pub(crate) privy: Arc<Privy>,
    pub(crate) mongo: Arc<MongoClient>,
}

impl AppState {
    pub fn new(privy: Privy, mongo: MongoClient) -> Self {
        Self {
            privy: Arc::new(privy),
            mongo: Arc::new(mongo),
        }
    }
}
