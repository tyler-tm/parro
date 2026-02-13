mod command;
mod error;
mod handler;
mod server;
mod storage;

use server::Server;

#[tokio::main]

async fn main() -> error::Result<()> {
    let server = Server::new()
        .await
        .map_err(|e| -> error::Error { format!("Error while starting server: {}", e).into() })?;

    server.run().await
}
