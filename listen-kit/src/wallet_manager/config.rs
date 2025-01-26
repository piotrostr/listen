use anyhow::Result;

#[derive(Clone)]
pub struct PrivyConfig {
    pub(crate) app_id: String,
    pub(crate) app_secret: String,
    pub(crate) verification_key: String,
}

impl PrivyConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            app_id: std::env::var("PRIVY_APP_ID")
                .expect("PRIVY_APP_ID is not set"),
            app_secret: std::env::var("PRIVY_APP_SECRET")
                .expect("PRIVY_APP_SECRET is not set"),
            verification_key: std::env::var("PRIVY_VERIFICATION_KEY")
                .expect("PRIVY_VERIFICATION_KEY is not set"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_privy_config_from_env() {
        PrivyConfig::from_env().unwrap();
    }
}
