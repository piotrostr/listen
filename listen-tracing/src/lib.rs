use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_tracing() {
    // Create an EnvFilter that reads from RUST_LOG with INFO as default
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Configure logging based on environment
    if std::env::var("IS_SYSTEMD_SERVICE").is_ok() {
        // Use systemd formatting when running as a service
        let journald_layer = tracing_journald::layer().expect("Failed to create journald layer");
        tracing_subscriber::registry()
            .with(journald_layer)
            .with(env_filter)
            .init();
    } else {
        // Use standard formatting for non-systemd environments
        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_target(true)
            .with_env_filter(env_filter)
            .init();
    }
}
