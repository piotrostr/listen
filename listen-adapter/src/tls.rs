#[cfg(feature = "tls")]
use rustls::{pki_types::PrivateKeyDer, ServerConfig};
#[cfg(feature = "tls")]
use rustls_pemfile::{certs, pkcs8_private_keys};
#[cfg(feature = "tls")]
use std::{fs::File, io::BufReader};

#[cfg(feature = "tls")]
pub fn load_rustls_config() -> std::io::Result<ServerConfig> {
    // Install the AWS-LC crypto provider as the default
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install AWS-LC crypto provider");

    // Load SSL certificate and private key files
    let cert_path = std::env::var("SSL_CERT_PATH").expect("SSL_CERT_PATH must be set");
    let key_path = std::env::var("SSL_KEY_PATH").expect("SSL_KEY_PATH must be set");

    let cert_file = &mut BufReader::new(File::open(cert_path)?);
    let key_file = &mut BufReader::new(File::open(key_path)?);

    let cert_chain = certs(cert_file)
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to load certificate");

    let mut keys = pkcs8_private_keys(key_file)
        .map(|key| key.map(PrivateKeyDer::Pkcs8))
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to load private key");

    // Exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    // Create SSL configuration
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .expect("Failed to create SSL config");

    Ok(config)
}

#[cfg(not(feature = "tls"))]
pub fn load_rustls_config() -> std::io::Result<()> {
    unimplemented!("TLS support is not enabled")
}
