#[cfg(feature = "server")]
use parro::Config;
#[cfg(feature = "server")]
use parro::server::Server;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> parro::error::Result<()> {
    let config = Config::from_env();
    let server = Server::new(config)
        .await
        .map_err(|e| -> parro::error::Error { format!("Error while starting server: {e}").into() })?;
    server.run().await
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!("The server binary requires the `server` feature.");
    std::process::exit(1);
}
