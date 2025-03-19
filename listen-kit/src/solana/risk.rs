use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskReport {
    risks: Vec<Risk>,
    score: u32,
    score_normalised: u32,
    #[serde(rename = "tokenProgram")]
    token_program: String,
    #[serde(rename = "tokenType")]
    token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Risk {
    description: String,
    level: String,
    name: String,
    score: u32,
    value: String,
}

pub async fn get_risk_report(mint: String) -> Result<serde_json::Value> {
    let url =
        format!("https://api.rugcheck.xyz/v1/tokens/{}/report/summary", mint);
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(serde_json::from_str(&body)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_risk_report() {
        let mint = "mMzPeKPgvnosRiVGDYWAgH8ArUsyxfRxYyE3vZ5pump";
        let report = get_risk_report(mint.to_string()).await.unwrap();
        println!("{:?}", report);
    }
}
