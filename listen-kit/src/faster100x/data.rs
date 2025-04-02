use crate::faster100x::types::{Faster100xData, Faster100xResponse};
use anyhow::{anyhow, Result};
use reqwest::Client;

pub async fn get_faster100x_data(
    token_address: &str,
) -> Result<Faster100xData> {
    tracing::debug!("Requesting data for token: {}", token_address);

    let client = Client::new();
    let url = "https://faster100x.com/api/trpc/embedded.getAnalyzeResult";
    let params = [
        ("batch", "1"),
        (
            "input",
            &serde_json::json!({
                "0": { "json": { "tokenAddress": token_address } }
            })
            .to_string(),
        ),
    ];

    // Make request and get response text
    let response = client
        .get(url)
        .query(&params)
        .send()
        .await
        .map_err(|e| anyhow!("[Faster100x] HTTP request error: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| anyhow!("[Faster100x] Error reading response: {}", e))?;

    tracing::debug!("[Faster100x] Raw response: {}", response_text);

    // Parse response
    let data: Vec<Faster100xResponse> = serde_json::from_str(&response_text)
        .map_err(|e| {
            anyhow!(
                "[Faster100x] JSON parsing error: {} - Response: {}",
                e,
                response_text
            )
        })?;

    let result = data
        .first()
        .ok_or_else(|| anyhow!("[Faster100x] Empty response from API"))?
        .result
        .data
        .json
        .clone();

    // Validate response status
    if result.status != "success" {
        return Err(anyhow!("[Faster100x] API error: {:?}", result.message));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_faster100x_data() {
        let data = get_faster100x_data(
            "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump",
        )
        .await;
        tracing::info!("{:?}", data);
    }
}
