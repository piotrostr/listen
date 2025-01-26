use crate::wallet_manager::config::PrivyConfig;
use crate::wallet_manager::WalletManager;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel;
use std::sync::Arc;

pub struct AppState {
    pub(crate) agent: Arc<Agent<CompletionModel>>,
    pub(crate) wallet_manager: Arc<WalletManager>,
}

impl AppState {
    pub fn new(agent: Agent<CompletionModel>) -> Self {
        Self {
            agent: agent.into(),
            wallet_manager: WalletManager::new(
                PrivyConfig::from_env().expect("Failed to load privy config"),
            )
            .into(),
        }
    }
}
