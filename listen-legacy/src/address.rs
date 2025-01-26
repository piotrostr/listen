use std::{
    fs::write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use log::info;
use solana_sdk::{signature::Keypair, signer::Signer};

pub async fn generate_custom_sol_address(
    prefixes: Vec<String>,
    found_flag: Arc<AtomicBool>,
) {
    info!("Starting search for {:?}", prefixes);
    let mut tries = 0;
    while !found_flag.load(Ordering::Relaxed) {
        let keypair = Keypair::new();
        if let Some(prefix) = prefixes.iter().find(|prefix| {
            keypair
                .pubkey()
                .to_string()
                .to_lowercase()
                .starts_with(prefix.as_str())
        }) {
            found_flag.store(true, Ordering::Relaxed);
            info!(
                "Match found for {}: {}",
                prefix,
                keypair.pubkey().to_string()
            );
            let mut id = [0u8; 64];
            id[..32].copy_from_slice(keypair.secret().as_bytes());
            id[32..].copy_from_slice(&keypair.pubkey().to_bytes());
            info!("id: {:#?}", id);
            write(
                format!("{}.json", prefix),
                serde_json::to_string(&Vec::from(&id)).expect("vec into json"),
            )
            .expect("write private key");
            break;
        }
        tries += 1;
        if tries % 100_000 == 0 {
            info!("tried {} so far", tries);
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_write_bytes() {
        let contents = [255u8; 64];
        super::write(
            "tmp.json",
            serde_json::to_string(&Vec::from(&contents)).unwrap(),
        )
        .unwrap();

        if std::fs::metadata("tmp.json").is_ok() {
            std::fs::remove_file("tmp.json").unwrap();
        }
    }
}
