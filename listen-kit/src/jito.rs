use log::info;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::{
    Encodable, EncodedTransaction, UiTransactionEncoding,
};
use std::cell::RefCell;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct JitoResponse {
    pub jsonrpc: String,
    pub result: String,
    pub id: i64,
}

// TODO support versioned transactions (jup 3+ multi-hops)
#[timed::timed(duration(printer = "info!"))]
pub async fn send_jito_tx(
    tx: Transaction,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let encoded_tx = match tx.encode(UiTransactionEncoding::Binary) {
        EncodedTransaction::LegacyBinary(b) => b,
        _ => return Err("Failed to encode transaction".into()),
    };

    let res = client
        .post("https://mainnet.block-engine.jito.wtf/api/v1/transactions")
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [encoded_tx]
        }))
        .send()
        .await
        .expect("send tx");

    let jito_response = res.json::<JitoResponse>().await?;

    Ok(jito_response.result)
}

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

#[inline(always)]
pub fn fast_random_0_to_7() -> u8 {
    RNG.with(|rng| rng.borrow_mut().gen_range(0..8))
}

pub fn get_jito_tip_pubkey() -> Pubkey {
    const PUBKEYS: [&str; 8] = [
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
        "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
        "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
        "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
        "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
        "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
        "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
        "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
    ];
    let index = fast_random_0_to_7();
    Pubkey::from_str(PUBKEYS[index as usize]).expect("parse tip pubkey")
}

#[cfg(test)]
mod tests {
    #[test]
    fn bench_get_jito_tip_pubkey() {
        for _ in 0..100 {
            let start = std::time::Instant::now();
            let _ = super::get_jito_tip_pubkey();
            let elapsed = start.elapsed();
            println!("elapsed: {:?}", elapsed);
        }
    }
}
