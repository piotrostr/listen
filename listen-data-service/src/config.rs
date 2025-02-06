use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub substreams: SubstreamsConfig,
    pub modules: ModulesConfig,
    pub surreal: SurrealConfig,
}

#[derive(Debug, Deserialize)]
pub struct SubstreamsConfig {
    pub endpoint: String,
    pub output_module_name: String,
    pub start_block: i64,
    pub end_block: u64,
}

#[derive(Debug, Deserialize)]
pub struct ModulesConfig {
    pub binary_path: String,
}

#[derive(Debug, Deserialize)]
pub struct SurrealConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
