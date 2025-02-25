use privy::Privy;
use std::sync::Arc;

pub struct AppState {
    pub(crate) privy: Arc<Privy>,
}

impl AppState {
    pub fn new(privy: Privy) -> Self {
        Self {
            privy: Arc::new(privy),
        }
    }
}
