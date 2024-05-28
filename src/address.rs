use std::{
    // fs::write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use log::info;
use solana_sdk::{signature::Keypair, signer::Signer};

pub async fn generate_custom_sol_address(prefixes: Vec<String>, found_flag: Arc<AtomicBool>) {
    let mut tries = 0;
    while !found_flag.load(Ordering::Relaxed) {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();
        let address = bs58::encode(pubkey.to_bytes()).into_string();
        if let Some(prefix) = prefixes
            .iter()
            .find(|prefix| address.starts_with(prefix.as_str()))
        {
            found_flag.store(true, Ordering::Relaxed);
            info!("Match found for {}: {}", prefix, address);
            // write into file with {prefix}.json
            // write(format!("{}.json", prefix), keypair.secret().as_bytes())
            //     .expect("write private key");
            info!("private key: {:#?}", keypair.secret());
            break;
        }
        tries += 1;
        if tries % 10_000 == 0 {
            info!("tried {} so far", tries);
        }
    }
}
