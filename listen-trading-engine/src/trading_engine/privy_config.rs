use anyhow::Result;

#[derive(Clone)]
pub struct PrivyConfig {
    pub(crate) app_id: String,
    pub(crate) app_secret: String,
}

fn must_get_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Missing env var: {}", name))
}

impl PrivyConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            app_id: must_get_env("PRIVY_APP_ID"),
            app_secret: must_get_env("PRIVY_APP_SECRET"),
        })
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
