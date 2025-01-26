#[cfg(feature = "http")]
use listen_kit::http::server::run_server;

#[cfg(feature = "http")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use listen_kit::agent::create_trader_agent;

    let agent = create_trader_agent()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    run_server(agent).await
}

#[cfg(not(feature = "http"))]
fn main() {
    println!("This binary requires the 'http' feature");
}
