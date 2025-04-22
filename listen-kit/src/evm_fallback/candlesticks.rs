use super::{map_chain_id_to_network, EvmFallback};
use crate::data::Candlestick;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap; // Added for timestamp conversion

// Helper function to map interval string to GeckoTerminal timeframe and aggregate
// Example interval formats: "1m", "5m", "15m", "1h", "4h", "1d"
pub fn map_interval_to_params(
    interval: &str,
) -> Result<(&'static str, &'static str)> {
    match interval {
        "1m" => Ok(("minute", "1")),
        "5m" => Ok(("minute", "5")),
        "15m" => Ok(("minute", "15")),
        "1h" => Ok(("hour", "1")),
        "4h" => Ok(("hour", "4")),
        "1d" => Ok(("day", "1")),
        _ => Err(anyhow!("Unsupported interval format: {}", interval)),
    }
}

// Structs for deserializing GeckoTerminal OHLCV API response
#[derive(Deserialize, Debug)]
pub struct GTOhlcvAttributes {
    // [timestamp(s), open, high, low, close, volume]
    ohlcv_list: Vec<[Value; 6]>,
}

#[derive(Deserialize, Debug)]
pub struct GTOhlcvData {
    attributes: GTOhlcvAttributes,
}

#[derive(Deserialize, Debug)]
pub struct GTCandlesticksResponse {
    data: GTOhlcvData,
}

impl EvmFallback {
    pub async fn fetch_candlesticks(
        &self,
        pool_address: &str,
        chain_id: u64,
        interval: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Candlestick>> {
        let network = map_chain_id_to_network(chain_id)?;
        let (timeframe, aggregate) = map_interval_to_params(interval)?;

        let mut url = format!(
            "{}/networks/{}/pools/{}/ohlcv/{}",
            self.base_url, network, pool_address, timeframe
        );

        // Build query parameters
        let mut query_params = HashMap::new();
        query_params.insert("aggregate".to_string(), aggregate.to_string());

        if let Some(limit) = limit {
            // API max limit is 1000
            query_params
                .insert("limit".to_string(), limit.min(1000).to_string());
        } else {
            // Default limit from API docs is 100
            query_params.insert("limit".to_string(), "100".to_string());
        }

        let query_string = query_params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&");

        if !query_string.is_empty() {
            url.push('?');
            url.push_str(&query_string);
        }

        let response = self
            .client
            .get(&url)
            .header(
                "Accept",
                format!("application/json;version={}", self.api_version),
            )
            .send()
            .await
            .context(format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(anyhow!(
                "GeckoTerminal API request failed for OHLCV ({}): {} - {}",
                url,
                status,
                error_text
            ));
        }

        let gt_candlesticks_resp = response
            .json::<GTCandlesticksResponse>()
            .await
            .context("Failed to deserialize GeckoTerminal OHLCV response")?;

        // Convert GTOhlcvData to Vec<Candlestick>
        let candlesticks = gt_candlesticks_resp
            .data
            .attributes
            .ohlcv_list
            .into_iter()
            .filter_map(|item| {
                // Expecting [timestamp, open, high, low, close, volume]
                if item.len() != 6 {
                    eprintln!(
                        "Warning: Received malformed OHLCV item: {:?}",
                        item
                    );
                    return None;
                }
                let timestamp_val = item[0].as_u64()?;
                let open_val = item[1].as_f64()?;
                let high_val = item[2].as_f64()?;
                let low_val = item[3].as_f64()?;
                let close_val = item[4].as_f64()?;
                let volume_val = item[5].as_f64()?;

                Some(Candlestick {
                    timestamp: timestamp_val,
                    open: open_val,
                    high: high_val,
                    low: low_val,
                    close: close_val,
                    volume: volume_val,
                })
            })
            .collect::<Vec<Candlestick>>();

        Ok(candlesticks)
    }
}
