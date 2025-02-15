use anyhow::Result;

#[derive(Clone)]
pub struct PrivyConfig {
    pub(crate) app_id: String,
    pub(crate) app_secret: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PrivyConfigError {
    #[error("[Privy] Missing required environment variable: {0}")]
    MissingEnvVar(&'static str),
}

impl PrivyConfig {
    pub fn from_env() -> Result<Self, PrivyConfigError> {
        let app_id = std::env::var("PRIVY_APP_ID")
            .map_err(|_| PrivyConfigError::MissingEnvVar("PRIVY_APP_ID"))?;

        let app_secret = std::env::var("PRIVY_APP_SECRET")
            .map_err(|_| PrivyConfigError::MissingEnvVar("PRIVY_APP_SECRET"))?;

        Ok(Self { app_id, app_secret })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privy_config_from_env() {
        PrivyConfig::from_env().unwrap();
    }
}
