use super::config::PrivyConfig;
use base64::{engine::general_purpose::STANDARD, Engine as _};

pub fn base64encode(data: &[u8]) -> String {
    STANDARD.encode(data)
}

pub fn create_privy_client(privy_config: &PrivyConfig) -> reqwest::Client {
    reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "privy-app-id",
                privy_config.app_id.parse().expect("Failed to parse header"),
            );
            headers.insert(
                "Content-Type",
                "application/json".parse().expect("Failed to parse header"),
            );
            headers.insert(
                "Authorization",
                format!(
                    "Basic {}",
                    base64encode(
                        format!("{}:{}", privy_config.app_id, privy_config.app_secret).as_bytes(),
                    )
                )
                .parse()
                .expect("Failed to parse header"),
            );
            headers
        })
        .build()
        .unwrap()
}
