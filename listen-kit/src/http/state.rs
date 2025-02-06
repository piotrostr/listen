use crate::wallet_manager::WalletManager;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel;
use std::sync::Arc;

pub struct AppState {
    #[cfg(feature = "solana")]
    pub(crate) solana_agent: Arc<Agent<CompletionModel>>,
    #[cfg(feature = "evm")]
    pub(crate) evm_agent: Arc<Agent<CompletionModel>>,
    pub(crate) wallet_manager: Arc<WalletManager>,
    pub(crate) omni_agent: Arc<Agent<CompletionModel>>,
}

pub struct AppStateBuilder {
    #[cfg(feature = "solana")]
    solana_agent: Option<Agent<CompletionModel>>,
    #[cfg(feature = "evm")]
    evm_agent: Option<Agent<CompletionModel>>,
    wallet_manager: Option<WalletManager>,
    omni_agent: Option<Agent<CompletionModel>>,
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppStateBuilder {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "solana")]
            solana_agent: None,
            #[cfg(feature = "evm")]
            evm_agent: None,
            wallet_manager: None,
            omni_agent: None,
        }
    }

    #[cfg(feature = "solana")]
    pub fn with_solana_agent(
        mut self,
        agent: Agent<CompletionModel>,
    ) -> Self {
        self.solana_agent = Some(agent);
        self
    }

    #[cfg(feature = "evm")]
    pub fn with_evm_agent(mut self, agent: Agent<CompletionModel>) -> Self {
        self.evm_agent = Some(agent);
        self
    }

    pub fn with_wallet_manager(
        mut self,
        wallet_manager: WalletManager,
    ) -> Self {
        self.wallet_manager = Some(wallet_manager);
        self
    }

    pub fn with_omni_agent(mut self, agent: Agent<CompletionModel>) -> Self {
        self.omni_agent = Some(agent);
        self
    }

    pub fn build(self) -> Result<AppState, &'static str> {
        Ok(AppState {
            #[cfg(feature = "solana")]
            solana_agent: Arc::new(self.solana_agent.ok_or(
                "Solana agent is required when solana feature is enabled",
            )?),
            #[cfg(feature = "evm")]
            evm_agent: Arc::new(self.evm_agent.ok_or(
                "EVM agent is required when evm feature is enabled",
            )?),
            wallet_manager: Arc::new(
                self.wallet_manager.ok_or("Wallet manager is required")?,
            ),
            omni_agent: Arc::new(
                self.omni_agent
                    .expect("omni agent is required with http feature"),
            ),
        })
    }
}

impl AppState {
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::default()
    }
}
